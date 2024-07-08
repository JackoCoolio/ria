```roc
filter : 'l List ('a a) -> ('a a -> 'b Bool) -> 'l List ('a a)
filter list func =
    when list is
        [] -> []
        [x : xs] ->
            # `func` is `'a a -> 'b Bool`, so `x` isn't evaluated here, we just
            # pass the possibly-effectful value into `func`. This means that  
            if func x then
                x : filter xs func
            else
                filter xs func
```

# if `a` is [Get -> a] a, maybe from a function `get_a : [Get -> a] a`

a_is_good : 'a a -> Bool

as = [get_a, get_a, get_a]

filter as a_is_good

# this is monomorphized into:

when as is
    [] -> []
    [x : xs] ->
        if a_is_good 
