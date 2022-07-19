## Info for RQ1

```json
{
    "binary": "id",
    "tpe": "original",
    "sections": [
        {
            "tpe": "data",
            "size": 0
        }
    ] ,
    "functions": [
        {
            "idx": 0,
            "number_instructions": 0,
            "instructions": [
                {
                    "idx": 1,
                    "instr": "i32.const 10",
                    "tpe": "i32.const",
                    "mutations": [
                        {
                            "tpe": "CodeMotion",
                            "canmutate": false
                        },
                        {
                            "tpe": "Peephole",
                            "canmutate": true,
                            "paths": 100000
                        }
                    ] 
                }
            ]
        }
    ]
}
```

## JSON for reduced

```json
{
    "binary": "id",
    "tpe": "reduced",
    "parent": "id",
    "functions": [
        {
            "idx": 0,
            "number_instructions": 0,
            "instructions": [
                {
                    "idx": 1,
                    "instr": "i32.const 10",
                    "tpe": "i32.const"
                }
            ]
        }
    ]
}
```

## JSON for mutation

```json
{
    "binary": "id",
    "tpe": "variant",
    "parent": "id",
    "functions": [
        {
            "idx": 0,
            "number_instructions": 0,
            "instructions": [
                {
                    "idx": 1,
                    "instr": "i32.const 10",
                    "tpe": "i32.const",
                    "mutations": [
                        {
                            "tpe": "CodeMotion",
                        },
                        {
                            "tpe": "Peephole",
                            "path": "a + b + d + c",
                            "original_dfg": "",
                            "mutated_dfg": "" 
                        }
                    ] 
                }
            ]
        }
    ]
}
```