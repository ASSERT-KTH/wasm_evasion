Obfuscation paper:
Differences:
- They do obfuscation at high level
    - If LLVM, only 70% can be evaluated
- They do 7 transformations only
- The distance metric is based on the cosine distance


# Our experiments:

## RQ1: To what extent are the Wasm in the wild mutable ?
Method:
- Take all Wasm in the wild: 8k (wasmbench)
    - Concerning: it is 1 year old, probably out of date
    - IDEA: create a bot for collecting all wasm in the wild
        - continuously:
            - Scrape the internet, hash and check for new, optimize, if new, report, if same, report as a variant.
        - Modify Chromium as the oracle to detect Wasm binaries
- Evaluate with wasm-mutate:
    - How can it be mutated:
        - High level: structure level
            - Removal of custom sections
            - Adding of bogus function
            - TODO, check the others in wasm-mutate
        - Low level:
            - Data:
                - Insert bogus data
            - Code:
                - egraphs mutation based
                    - 
                - code motion based

```
for b in binaries:
    # number of functions, number of instructions per function, profiling of instructions, type of Wasm (1.0 or 2.0), existing sections
    meta = meta(b)
    
    meta2 = wasm-mutate-meta(b)
    # Per section, which mutator can be applied
    # Per function, per instruction -> How many can be mutated with the current #egraph
        # Up to limit M -> How many path (different Wasm binaries) can be #generated
            # Check preservation, How many are different according to: wasmtime and V8

```
        
## RQ2: What are the most used techniques to detect cryptomalwares ?
- Create features map
- Use commercial antiviruses and check their techniques
    - VirusTotal

## RQ3: How to bypass classifiers ?
- False negatives
- False positives
- What is the minimum path to take ?
- TODO, propose a new metric based in egraph path
- Use of case with MINOS

## RQ4: What is the map between obfuscation technique and the feature to break ?

## RQ5: 