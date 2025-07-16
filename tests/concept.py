# How our record will look like
type Record = { lang: str, time: int};

# implement methods for the record
from Record def |>
    def method(s: int, i: int ) -> int = 
        s + i

# how functions will be constructed
def foofoo() -> list =
    let value = Record {lang: "choco", time: 2 }
    let x =
        if value.time == 2:
            4+4
        else:
            0

    # comment
    match i with 
        | 1 -> something()
        | 2 -> value
        | 2 -> 100
        | _ -> 0 

    # how arrays would most likely look like
    [1,2,3,4]

