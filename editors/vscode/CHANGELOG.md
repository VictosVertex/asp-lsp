## [0.0.9] - 2024-07-21
### Fixed
- Fixed a crash that happened when not enough parameter descriptions were provided
- Fixed a crash that happened when dealing with `optcondition`s in disjunctions
- Fixed parameter specific hover descriptions depending on the order they are provided in

### Breaking Changes
- Parameter in parameter descriptions inside `docstrings` now require `- ` in front of them

    **previously**
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

    **now**
    ```
    %*
    #example_predicate(A,B,C).

    This is an example predicate used for the illustration of doc strings.

    #parameters
    - A : The first argument/parameter of the predicate.
    - B : Another parameter of this example predicate.
    - C : The last parameter in this example.
    *%
    ```

### Added
- You may now add additional info after the parameter descriptions by inserting an empty line followed by your additions
    ```
    %*
    #example_predicate(A,B,C).

    This is an example predicate used for the illustration of doc strings.

    #parameters
    - A : The first argument/parameter of the predicate.
    - B : Another parameter of this example predicate.
    - C : The last parameter in this example.

    Some extra stuff you want to tell your readers.
    *%
    ```