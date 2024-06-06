# asp-lsp
A VSCode extension for [asp-lsp](https://github.com/VictosVertex/asp-lsp), a language server for answer set programming.

## Features
- [Code Completion](#code-completion) for predicates and directives
- [Hover](#hover) feature for predicates using [`doc strings`](#doc-strings)
- [Diagnostics](#diagnostics) showing syntax errors
- [Safety](#safety) showing unsafe variables in statements
- [Goto](#goto) definitions and references



## Installation
1. Open Visual Studio Code.
2. Go to the Extensions view by clicking on the Extensions icon in the Activity Bar on the side of the window or by pressing `Ctrl+Shift+X`.
3. Search for `asp-lsp`.
4. Click `Install` to install the extension.

## How To Use
1. Open a `.lp` file or create a new one in your workspace.
2. The language server should start automatically with all available features

## Feature Details
### Code Completion
The language server currently supports keyword completions for directives such as `#show`, `#const` and `#minimize` as well as completion for previously declared predicates.

Given a predicate such as:
```
example(1,2,3,4).
```

When trying to insert it elsewhere the code completion first offers a selection of possible completion candidates.

![predicate completion](https://github.com/VictosVertex/asp-lsp-assets/blob/main/vscode_extension/predicate-completion.png?raw=true)

After selecting a predicate, the completion inserts a fitting template that allows to quickly insert arguments by using `TAB` to jump from one to the next.

![predicate completion template](https://github.com/VictosVertex/asp-lsp-assets/blob/main/vscode_extension/predicate-completion-template.png?raw=true)

### Hover
In order to use the `hover` feature for predicates one first has to write `doc strings` that provide the wanted information.
#### Doc Strings
To define a `doc string` for a predicate, first a multi-line comment using `%*` and `*%` has to be created. The first content of the comment has to be a `#` followed by the predicate's full signature (including the dot!). It is advised to follow the signature with a general description of the predicate. Lastly `#parameters` on a single line opens up the parameter descriptions. Each parameters is described by their name followed by `:` and a description.

A full example may look as follows:
```
%*
#example_predicate(A,B,C).

This is an example predicate used for the illustration of doc strings.

#parameters
    A : The first argument/parameter of the predicate.
    B : Another parameter of this example predicate.
    C : The last parameter in this example.
*%
```

After this doc string is defined anywhere in the `.lp` file, simply hovering over any `example_predicate/3` will show the information.

![hover predicate](https://github.com/VictosVertex/asp-lsp-assets/blob/main/vscode_extension/hover-predicate.png?raw=true)

Hovering over an argument/parameter shows the corresponding informaton.

![hover predicate argument](https://github.com/VictosVertex/asp-lsp-assets/blob/main/vscode_extension/hover-argument.png?raw=true)

### Diagnostics
The language server is also capable of finding syntax errors such as missing dots after the declaration of statements.

![diagnostics](https://github.com/VictosVertex/asp-lsp-assets/blob/main/vscode_extension/missing-dot.png?raw=true)
### Safety
When defining rules, the language server will continuously check whether the variables used in the statement are safe or not.

![safety](https://github.com/VictosVertex/asp-lsp-assets/blob/main/vscode_extension/unsafe.png?raw=true)

### Goto
Since ASP allows for multiple declarations of the same predicate, **goto definition** shows all occurrences of the predicate in the **head**. Similarly, **goto reference** shows all occurrences of the predicate in the **body**.

## Issues, Requests Or Questions
If you have any issues, requests, questions or suggestions, feel free to contact me directly or open an [issue](https://github.com/VictosVertex/asp-lsp/issues) on github.