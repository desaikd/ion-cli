type::{
 name: nested_struct,
 fields: {
    A: string,
    B: int,
    C: {
        fields: {
            D: { element: int },
            E: bool
        }
    }
 }
}