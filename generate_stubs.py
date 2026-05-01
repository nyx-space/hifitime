import argparse
import ast
import importlib
import inspect
import logging
import re
import subprocess
from functools import reduce
from typing import Any, Dict, List, Mapping, Optional, Set, Tuple, Union


def path_to_type(*elements: str) -> ast.AST:
    base: ast.AST = ast.Name(id=elements[0], ctx=ast.Load())
    for e in elements[1:]:
        base = ast.Attribute(value=base, attr=e, ctx=ast.Load())
    return base


OBJECT_MEMBERS = dict(inspect.getmembers(object))
BUILTINS: Dict[str, Union[None, Tuple[List[ast.AST], ast.AST]]] = {
    "__annotations__": None,
    "__bool__": ([], path_to_type("bool")),
    "__bytes__": ([], path_to_type("bytes")),
    "__class__": None,
    "__contains__": ([path_to_type("typing", "Any")], path_to_type("bool")),
    "__del__": None,
    "__delattr__": ([path_to_type("str")], path_to_type("None")),
    "__delitem__": ([path_to_type("typing", "Any")], path_to_type("None")),
    "__dict__": None,
    "__dir__": None,
    "__doc__": None,
    "__eq__": ([path_to_type("typing", "Any")], path_to_type("bool")),
    "__format__": ([path_to_type("str")], path_to_type("str")),
    "__ge__": ([path_to_type("typing", "Any")], path_to_type("bool")),
    "__getattribute__": ([path_to_type("str")], path_to_type("typing", "Any")),
    "__getitem__": ([path_to_type("typing", "Any")], path_to_type("typing", "Any")),
    "__gt__": ([path_to_type("typing", "Any")], path_to_type("bool")),
    "__hash__": ([], path_to_type("int")),
    "__init__": ([], path_to_type("None")),
    "__init_subclass__": None,
    "__iter__": ([], path_to_type("typing", "Any")),
    "__le__": ([path_to_type("typing", "Any")], path_to_type("bool")),
    "__len__": ([], path_to_type("int")),
    "__lt__": ([path_to_type("typing", "Any")], path_to_type("bool")),
    "__module__": None,
    "__ne__": ([path_to_type("typing", "Any")], path_to_type("bool")),
    "__new__": None,
    "__next__": ([], path_to_type("typing", "Any")),
    "__int__": ([], path_to_type("None")),
    "__reduce__": None,
    "__reduce_ex__": None,
    "__repr__": ([], path_to_type("str")),
    "__setattr__": (
        [path_to_type("str"), path_to_type("typing", "Any")],
        path_to_type("None"),
    ),
    "__setitem__": (
        [path_to_type("typing", "Any"), path_to_type("typing", "Any")],
        path_to_type("None"),
    ),
    "__sizeof__": None,
    "__str__": ([], path_to_type("str")),
    "__subclasshook__": None,
    "__add__": ([path_to_type("typing", "Any")], path_to_type("typing", "Any")),
    "__sub__": ([path_to_type("typing", "Any")], path_to_type("typing", "Any")),
    "__mul__": ([path_to_type("typing", "Any")], path_to_type("typing", "Any")),
    "__truediv__": ([path_to_type("typing", "Any")], path_to_type("typing", "Any")),
    "__div__": ([path_to_type("typing", "Any")], path_to_type("typing", "Any")),
    "__radd__": ([path_to_type("typing", "Any")], path_to_type("typing", "Any")),
    "__rsub__": ([path_to_type("typing", "Any")], path_to_type("typing", "Any")),
    "__rmul__": ([path_to_type("typing", "Any")], path_to_type("typing", "Any")),
    "__rtruediv__": ([path_to_type("typing", "Any")], path_to_type("typing", "Any")),
    "__setstate__": ([path_to_type("typing", "Any")], path_to_type("None")),
    "__getnewargs__": ([], path_to_type("typing", "Tuple")),
}

TYPE_MAPPING = {
    "numpy.array": "numpy.ndarray",
    "numpy.ndarray": "numpy.ndarray",
    "np.ndarray": "numpy.ndarray",
    "np.array": "numpy.ndarray",
    "tuples": "typing.Tuple",
}

def module_stubs(module: Any) -> ast.Module:
    types_to_import = {"typing"}
    classes = []
    functions = []
    for member_name, member_value in inspect.getmembers(module):
        element_path = [module.__name__, member_name]
        if member_name.startswith("__"):
            pass
        elif inspect.isclass(member_value):
            classes.append(
                class_stubs(member_name, member_value, element_path, types_to_import)
            )
        elif inspect.isbuiltin(member_value) or inspect.isroutine(member_value):
            functions.append(
                function_stub(
                    member_name,
                    member_value,
                    element_path,
                    types_to_import,
                    in_class=False,
                )
            )
        else:
            logging.warning(f"Unsupported root construction {member_name}")
    # Add metadata constants if they exist in the module
    for meta in ["__author__", "__version__"]:
        if hasattr(module, meta):
            functions.append(
                ast.AnnAssign(
                    target=ast.Name(id=meta, ctx=ast.Store()),
                    annotation=ast.Name(id="str", ctx=ast.Load()),
                    value=ast.Constant(getattr(module, meta)),
                    simple=1,
                )
            )

    # Resolve all types to import
    imports = [
        ast.ImportFrom(module="__future__", names=[ast.alias(name="annotations")], level=0)
    ]
    for t in sorted(types_to_import):
        if t == "typing":
            imports.append(ast.Import(names=[ast.alias(name=t)]))
            continue

        if t == "numpy":
             imports.append(ast.Import(names=[ast.alias(name="numpy")]))
             continue

        if t.startswith("anise."):
            current_module = module.__name__
            if current_module == t:
                 continue

            parts = t.split(".")
            # e.g. from anise import time
            imports.append(ast.ImportFrom(module=parts[0], names=[ast.alias(name=parts[1])], level=0))
        elif t == "anise":
             # Avoid self-import
             if module.__name__ != "anise":
                  imports.append(ast.Import(names=[ast.alias(name="anise")]))
        else:
            imports.append(ast.Import(names=[ast.alias(name=t)]))

    return ast.Module(
        body=imports + classes + functions,
        type_ignores=[],
    )


def class_stubs(
    cls_name: str, cls_def: Any, element_path: List[str], types_to_import: Set[str]
) -> ast.ClassDef:
    attributes: List[ast.AST] = []
    methods: List[ast.AST] = []
    magic_methods: List[ast.AST] = []
    constants: List[ast.AST] = []

    for member_name, member_value in inspect.getmembers(cls_def):
        current_element_path = [*element_path, member_name]
        # Inside the loop in class_stubs
        if member_name in ("__init__", "__new__") and "Error" not in cls_name:
            try:
                actual_member = cls_def if member_name == "__new__" else member_value
                methods.append(
                    function_stub(
                        member_name,
                        actual_member,
                        current_element_path,
                        types_to_import,
                        in_class=True,
                        cls_def=cls_def
                    )
                )
            except Exception:
                continue
        elif (
            member_value == OBJECT_MEMBERS.get(member_name)
            or BUILTINS.get(member_name, ()) is None
        ):
            pass
        elif inspect.isdatadescriptor(member_value):
            attributes.extend(
                data_descriptor_stub(
                    member_name, member_value, current_element_path, types_to_import
                )
            )
        elif inspect.isroutine(member_value) or inspect.isbuiltin(member_value):
            (magic_methods if member_name.startswith("__") else methods).append(
                function_stub(
                    member_name,
                    member_value,
                    current_element_path,
                    types_to_import,
                    in_class=True,
                    cls_def=cls_def
                )
            )
        elif member_name == "__match_args__":
            constants.append(
                ast.AnnAssign(
                    target=ast.Name(id=member_name, ctx=ast.Store()),
                    annotation=ast.Subscript(
                        value=path_to_type("tuple"),
                        slice=ast.Tuple(
                            elts=[path_to_type("str"), ast.Ellipsis()], ctx=ast.Load()
                        ),
                        ctx=ast.Load(),
                    ),
                    value=ast.Constant(member_value),
                    simple=1,
                )
            )
        elif member_value is not None and not member_name.startswith("_"):
            try:
                annotation = concatenated_path_to_type(
                        member_value.__class__.__name__, element_path, types_to_import
                    )
                constants.append(
                    ast.AnnAssign(
                        target=ast.Name(id=member_name, ctx=ast.Store()),
                        annotation=annotation,
                        value=ast.Ellipsis(),
                        simple=1,
                    )
                )
            except Exception:
                pass
        else:
            pass

    doc = inspect.getdoc(cls_def)
    doc_comment = build_doc_comment(doc) if doc else None
    return ast.ClassDef(
        cls_name,
        bases=[],
        keywords=[],
        body=(
            ([doc_comment] if doc_comment else [])
            + attributes
            + methods
            + magic_methods
            + constants
        )
        or [ast.Ellipsis()],
        decorator_list=[path_to_type("typing", "final")],
    )


def data_descriptor_stub(
    data_desc_name: str,
    data_desc_def: Any,
    element_path: List[str],
    types_to_import: Set[str],
) -> Union[Tuple[ast.AnnAssign, ast.Expr], Tuple[ast.AnnAssign]]:
    annotation = None
    doc_comment = None

    doc = inspect.getdoc(data_desc_def)
    if doc is not None:
        try:
            annotation = returns_stub(data_desc_name, doc, element_path, types_to_import)
        except Exception:
            pass
        m = re.findall(r"^ *:return: *(.*) *$", doc, re.MULTILINE)
        if len(m) == 1:
            doc_comment = m[0]
        elif len(m) > 1:
            raise ValueError(
                f"Multiple return annotations found with :return: in {'.'.join(element_path)} documentation"
            )

    assign = ast.AnnAssign(
        target=ast.Name(id=data_desc_name, ctx=ast.Store()),
        annotation=annotation or path_to_type("typing", "Any"),
        simple=1,
    )
    doc_comment = build_doc_comment(doc_comment) if doc_comment else None
    return (assign, doc_comment) if doc_comment else (assign,)


def function_stub(
    fn_name: str,
    fn_def: Any,
    element_path: List[str],
    types_to_import: Set[str],
    *,
    in_class: bool,
    cls_def: Any = None
) -> ast.FunctionDef:
    body: List[ast.AST] = []
    doc = inspect.getdoc(fn_def)
    if doc is not None:
        doc_comment = build_doc_comment(doc)
        if doc_comment is not None:
            body.append(doc_comment)

    decorator_list = []
    is_static = False
    is_class = False

    if in_class:
        try:
            # Check if it's explicitly classmethod or staticmethod
            actual_fn = getattr(cls_def, fn_name, None)
            if isinstance(actual_fn, classmethod):
                 decorator_list.append(ast.Name(id="classmethod", ctx=ast.Load()))
                 is_class = True
            elif isinstance(actual_fn, staticmethod):
                 decorator_list.append(ast.Name(id="staticmethod", ctx=ast.Load()))
                 is_static = True
            else:
                try:
                    sig = inspect.signature(fn_def)
                    params = list(sig.parameters.values())
                    if params and params[0].name == "cls":
                        decorator_list.append(ast.Name(id="classmethod", ctx=ast.Load()))
                        is_class = True
                    elif (
                        in_class
                        and hasattr(fn_def, "__self__")
                        and not isinstance(fn_def.__self__, type)
                    ):
                        # Standard instance method bound
                        pass
                    elif in_class and hasattr(fn_def, "__self__"):
                        decorator_list.append(ast.Name(id="staticmethod", ctx=ast.Load()))
                        is_static = True
                except ValueError:
                    # Fallback for builtins in exception classes
                    if "Error" in element_path[-2]:
                         pass # default to instance method
        except Exception:
            pass

    print(f"Documenting {fn_name}")

    return ast.FunctionDef(
        fn_name,
        arguments_stub(fn_name, fn_def, doc or "", element_path, types_to_import, in_class, is_static, is_class),
        body or [ast.Ellipsis()],
        decorator_list=decorator_list,
        returns=(
            returns_stub(fn_name, doc or "", element_path, types_to_import, in_class)
        ),
        lineno=0,
    )


def arguments_stub(
    callable_name: str,
    callable_def: Any,
    doc: str,
    element_path: List[str],
    types_to_import: Set[str],
    in_class: bool,
    is_static: bool,
    is_class: bool,
) -> ast.arguments:
    if "Error" in element_path[1]:
        # Don't document errors
        return ast.arguments(posonlyargs=[], args=[], defaults=[], kwonlyargs=[])

    builtin = BUILTINS.get(callable_name)
    try:
        sig = inspect.signature(callable_def)
        real_parameters: Mapping[str, inspect.Parameter] = sig.parameters
    except Exception:
        # Fallback for builtins without signatures
        args = []
        if in_class:
            if is_static:
                 pass
            elif callable_name == "__new__" or is_class:
                args.append(ast.arg(arg="cls", annotation=None))
            else:
                args.append(ast.arg(arg="self", annotation=None))

        # If we have a BUILTIN entry, use it for arguments too
        if isinstance(builtin, tuple):
             for i, t in enumerate(builtin[0]):
                  args.append(ast.arg(arg=f"arg{i}", annotation=t))
             return ast.arguments(posonlyargs=[], args=args, defaults=[], kwonlyargs=[])

        return ast.arguments(
            posonlyargs=[],
            args=args,
            vararg=ast.arg(arg="args", annotation=path_to_type("typing", "Any")),
            kwonlyargs=[],
            kw_defaults=[],
            defaults=[],
            kwarg=ast.arg(arg="kwargs", annotation=path_to_type("typing", "Any")),
        )

    modified_parameters = dict(real_parameters)
    if in_class:
        if callable_name == "__init__":
            if "self" not in modified_parameters:
                modified_parameters = {
                    "self": inspect.Parameter("self", inspect.Parameter.POSITIONAL_ONLY),
                    **modified_parameters,
                }
        elif callable_name == "__new__":
            if "cls" not in modified_parameters:
                modified_parameters = {
                    "cls": inspect.Parameter("cls", inspect.Parameter.POSITIONAL_ONLY),
                    **modified_parameters,
                }
        elif not is_static and not any(p.name in ("self", "cls") for p in real_parameters.values()):
             if is_class:
                  modified_parameters = {
                    "cls": inspect.Parameter("cls", inspect.Parameter.POSITIONAL_ONLY),
                    **modified_parameters,
                }
             else:
                  modified_parameters = {
                    "self": inspect.Parameter("self", inspect.Parameter.POSITIONAL_ONLY),
                    **modified_parameters,
                }

    parsed_param_types = {}
    optional_params = set()

    # Types for magic functions types
    if isinstance(builtin, tuple):
        param_names = list(modified_parameters.keys())
        if param_names and param_names[0] in ("self", "cls"):
            del param_names[0]
        for name, t in zip(param_names, builtin[0]):
            parsed_param_types[name] = t

    elif callable_name in [
        "__add__",
        "__sub__",
        "__div__",
        "__mul__",
        "__radd__",
        "__rsub__",
        "__rdiv__",
        "__rmul__",
    ]:
        return ast.arguments(posonlyargs=[], args=[], defaults=[], kwonlyargs=[])

    # Types from comment
    for match in re.findall(
        r"^ *:type *([a-zA-Z0-9_]+): ([^\n]*) *$", doc, re.MULTILINE
    ):
        if match[0] not in modified_parameters:
            logging.warning(
                f"The parameter {match[0]} of {'.'.join(element_path)} "
                "is defined in the documentation but not in the function signature"
            )
            continue
        type_str = match[1]
        if type_str.endswith(", optional"):
            optional_params.add(match[0])
            type_str = type_str[:-10]
        parsed_param_types[match[0]] = convert_type_from_doc(
            type_str, element_path, types_to_import
        )

    # we parse the parameters
    posonlyargs = []
    args = []
    vararg = None
    kwonlyargs = []
    kw_defaults = []
    kwarg = None
    defaults = []
    for param in modified_parameters.values():
        if param.name in ("self", "cls"):
            param_ast = ast.arg(arg=param.name, annotation=None)
        elif param.name not in parsed_param_types:
            logging.warning(
                f"The parameter {param.name} of {'.'.join(element_path)} "
                "has no type definition in the function documentation"
            )
            param_ast = ast.arg(arg=param.name, annotation=path_to_type("typing", "Any"))
        else:
            annotation = parsed_param_types.get(param.name)
            if param.name in optional_params or param.default != param.empty:
                annotation = ast.Subscript(
                    value=path_to_type("typing", "Optional"),
                    slice=annotation,
                    ctx=ast.Load()
                )
                types_to_import.add("typing")
            param_ast = ast.arg(arg=param.name, annotation=annotation)

        default_ast = None
        if param.default != param.empty:
            default_ast = ast.Constant(param.default)

        if param.kind == param.POSITIONAL_ONLY:
            args.append(param_ast)
        elif param.kind == param.POSITIONAL_OR_KEYWORD:
            args.append(param_ast)
            defaults.append(default_ast)
        elif param.kind == param.VAR_POSITIONAL:
            vararg = param_ast
        elif param.kind == param.KEYWORD_ONLY:
            kwonlyargs.append(param_ast)
            kw_defaults.append(default_ast)
        elif param.kind == param.VAR_KEYWORD:
            kwarg = param_ast

    return ast.arguments(
        posonlyargs=posonlyargs,
        args=args,
        vararg=vararg,
        kwonlyargs=kwonlyargs,
        kw_defaults=kw_defaults,
        defaults=defaults,
        kwarg=kwarg,
    )


def returns_stub(
    callable_name: str, doc: str, element_path: List[str], types_to_import: Set[str], in_class: bool = False
) -> Optional[ast.AST]:
    if "Error" in element_path[1]:
        # Don't document errors
        return

    if callable_name in [
        "__add__",
        "__sub__",
        "__div__",
        "__mul__",
        "__radd__",
        "__rsub__",
        "__rdiv__",
        "__rmul__",
    ]:
        return

    if callable_name == "__new__":
         # Returns the class itself
         if len(element_path) >= 2:
              return path_to_type(element_path[-2])
         return path_to_type("typing", "Any")

    m = re.findall(r"^ *:rtype: *([^\n]*) *$", doc, re.MULTILINE)
    if len(m) == 0:
        builtin = BUILTINS.get(callable_name)
        if isinstance(builtin, tuple) and builtin[1] is not None:
            return builtin[1]
        return path_to_type("typing", "Any")
    if len(m) > 1:
        raise ValueError(
            f"Multiple return type annotations found with :rtype: for {'.'.join(element_path)}"
        )
    return convert_type_from_doc(m[0], element_path, types_to_import)


def convert_type_from_doc(
    type_str: str, element_path: List[str], types_to_import: Set[str]
) -> ast.AST:
    type_str = type_str.strip()
    return parse_type_to_ast(type_str, element_path, types_to_import)


def parse_type_to_ast(
    type_str: str, element_path: List[str], types_to_import: Set[str]
) -> ast.AST:
    # Resolve known types
    if type_str in TYPE_MAPPING:
        type_str = TYPE_MAPPING[type_str]

    # If it's a type in the current module, we can strip the module name
    current_module = element_path[0]
    if type_str.startswith(current_module + "."):
        remainder = type_str[len(current_module)+1:]
        if "." not in remainder:
             type_str = remainder

    # let's tokenize
    tokens = []
    current_token = ""
    for c in type_str:
        if (
            "a" <= c <= "z"
            or "A" <= c <= "Z"
            or "0" <= c <= "9"
            or c == "_"
            or c == "."
        ):
            current_token += c
        else:
            if current_token:
                tokens.append(current_token)
            current_token = ""
            if c != " ":
                tokens.append(c)
    if current_token:
        tokens.append(current_token)

    # let's first parse nested parenthesis
    stack: List[List[Any]] = [[]]
    for token in tokens:
        if token == "[":
            children: List[str] = []
            stack[-1].append(children)
            stack.append(children)
        elif token == "]":
            stack.pop()
        else:
            stack[-1].append(token)

    def parse_sequence(sequence: List[Any]) -> ast.AST:
        # 1. Handle commas first: split the sequence into distinct arguments
        args: List[List[Any]] = [[]]
        for e in sequence:
            if e == ",":
                args.append([])
            else:
                args[-1].append(e)

        # Filter out empty args (e.g. trailing commas)
        args = [a for a in args if a]

        # 2. If there are multiple arguments (separated by commas),
        # we return an ast.Tuple (this handles tuple[int, int, ...])
        if len(args) > 1:
            return ast.Tuple(elts=[parse_sequence(arg) for arg in args], ctx=ast.Load())

        # 3. Existing logic for "or" (Union types) within a single argument
        actual_sequence = args[0]
        or_groups: List[List[Any]] = [[]]
        for e in actual_sequence:
            if e == "or":
                or_groups.append([])
            else:
                or_groups[-1].append(e)

        new_elements: List[ast.AST] = []
        for group in or_groups:
            if len(group) == 1 and isinstance(group[0], str):
                # Standard type: int
                new_elements.append(
                    concatenated_path_to_type(group[0], element_path, types_to_import)
                )
            elif (
                len(group) == 2
                and isinstance(group[0], str)
                and isinstance(group[1], list)
            ):
                # Nested type: list[int] or tuple[...]
                new_elements.append(
                    ast.Subscript(
                        value=concatenated_path_to_type(
                            group[0], element_path, types_to_import
                        ),
                        slice=parse_sequence(group[1]),
                        ctx=ast.Load(),
                    )
                )
            else:
                raise ValueError(
                    f"Not able to parse type fragment '{group}' used by {'.'.join(element_path)}"
                )

        if len(new_elements) == 1:
            return new_elements[0]

        return reduce(
            lambda left, right: ast.BinOp(left=left, op=ast.BitOr(), right=right),
            new_elements,
        )

    return parse_sequence(stack[0])


def concatenated_path_to_type(
    path: str, element_path: List[str], types_to_import: Set[str]
) -> ast.AST:
    # Resolve known types
    if path in TYPE_MAPPING:
        path = TYPE_MAPPING[path]

    if path == "numpy.ndarray":
         types_to_import.add("numpy")
         return path_to_type("numpy", "ndarray")

    parts = path.split(".")

    current_module = element_path[0]

    if path.startswith("anise."):
         # Check if it's in our module
         if path.startswith(current_module + "."):
              remainder = path[len(current_module)+1:]
              if "." not in remainder:
                   # Local type
                   return path_to_type(remainder)

         subparts = path.split(".")
         if len(subparts) >= 2:
              # from anise import time -> time.Epoch
              types_to_import.add(".".join(subparts[:2]))
              return path_to_type(*subparts[1:])

    # If it's a type in the current module, we can strip the module name
    if len(parts) > 1:
        current_module_parts = current_module.split(".")
        if parts[:len(current_module_parts)] == current_module_parts:
            parts = parts[len(current_module_parts):]
            if len(parts) == 1:
                 return path_to_type(*parts)
        else:
            types_to_import.add(".".join(parts[:-1]))

    return path_to_type(*parts)


def build_doc_comment(doc: str) -> Optional[ast.Expr]:
    lines = [line.strip() for line in doc.split("\n")]
    clean_lines = []
    for line in lines:
        if line.startswith((":type", ":rtype")):
            continue
        clean_lines.append(line)
    text = "\n".join(clean_lines).strip()
    return ast.Expr(value=ast.Constant(text)) if text else None


def format_with_ruff(file: str) -> None:
    subprocess.check_call(["python", "-m", "ruff", "format", file])


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Extract Python type stub from a python module."
    )
    parser.add_argument(
        "module_name", help="Name of the Python module for which generate stubs"
    )
    parser.add_argument(
        "out",
        help="Name of the Python stub file to write to",
        type=argparse.FileType("wt"),
    )
    parser.add_argument(
        "--ruff", help="Formats the generated stubs using Ruff", action="store_true"
    )
    args = parser.parse_args()
    stub_content = ast.unparse(module_stubs(importlib.import_module(args.module_name)))
    args.out.write(stub_content)
    if args.ruff:
        format_with_ruff(args.out.name)
