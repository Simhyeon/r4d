# NOT YET

1. There is something not working in mac.r4d but not reproducible in the folder... what?
    - After process exited, logger panics and I don't know why.
    - What I tried but no success
        - Simple exit call
        - Exit call inside if macro

2. * [ ] Fix parsing rule
    * [ ] Parsing rule conflicts with some regex expressions
    ``` 
    $regexpr(in_paren,\\(([^)]+)\\)) 
    ```

    * [ ] Assert doesn't check value correctly. Yet this looks like parsing
      error. No..
    ```
    $assert(\*(*\,\*(*\)
    = Assert requires two arguments
    ```

3. * [-] Dryrun doesn't print log positions well. weird...
      - Simple demo doesn't reproduce this... what?

# SOLVED

# POSSIBLY SOLVED

1. Counter was acting strange.

    `args.is_empty()` -> THis line leaves empty string with 1 sized vector
    For e.g. counter emits very peculiar error when no argumen was given. Fix all
    of them. God damn it.
    I changed parsing logic. Yet keep an eye for this variants.

2. * [-] Dryrun doesn't print log positions well. weird...
      - Simple demo doesn't reproduce this... what?

3. 
