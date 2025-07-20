# ChocoML: A Simple ML Compiler 

## Why? What's wrong with you? 
Creating and working on compilers has sharpened my skills beyond my expectations and given me a fresh perspective on software engineering.
I honestly love learning about Compilers, it's been a blessing to work on. 

Here's a list of things I plan on implementing:
    - **Garbage Collector**: *Just for sake of learning*
    - **Instruction Scheduler**: HUUGGGEE maybe...not entirely sure if I want to add that.
    - **Graph Coloring**: I have linear scan implemented, want to try out Graph Coloring.

## Status
Current idea for the Design of the language:
```py
# How our record will look like
type Record = { lang: str, time: int}

# implement methods for the record
from Record def |>
    def method(s: int, i: int) -> int = 
        return s + i

def test() -> None = print("something")

# how functions will be constructed
def foofoo() -> list =
    let value = Record {lang: "choco", time: 2 }
    let x =
        if value.time == 2:
            4+4
        else:
            0

    let x = 0
    let j = 1
    if x not j:
        return [j, x]

    # comment
    match i with 
        | 1 -> something()
        | 2 -> value
        | 2 -> [1,2] 
        | _ -> [0] 

    # how arrays would most likely look like
    array = [1,2,3,4] 

    return array 
```

More fun...Under heavy construction 🚧
