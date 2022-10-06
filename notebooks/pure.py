# Load those Wasm from the dataset with producers information
import json
import pandas as pd

#metadata = open("../RQ1/filtered-binaries-metadata_2/filtered.pretty.json", "r").read()
metadata = open("../RQ1/all-binaries-metadata/all.pretty.json", "r").read()

metadata = json.loads(metadata)

d = {
    'wasm': [],
    'producer': [],
    'language': [],
    'tool': [],
    'tool_version': []
}

for k in metadata:
    if 'producers' in metadata[k]:
        if metadata[k]['producers']:
            prod = metadata[k]['producers']
            if 'language' in prod:
                d['language'].append(list(prod['language'].keys())[0])
            else:
                d['language'].append(None)
                
            if 'processed-by' in prod:
                processor = prod['processed-by']
                # do a preprocessing here, to select the tool and the version only
                d['producer'].append(prod['processed-by'].__str__())
                tool = list(prod['processed-by'].keys())[0]
                #if len(processor) > 1:
                #    print(processor)
                    
                    
                d['tool'].append(tool)
                
                # getting version
                if tool == 'rustc':
                    # the version is the next 4-(nightly) characters
                    version = processor[tool][:8]
                    d['tool_version'].append(f"{tool} {version}")
                elif tool == 'clang':
                    
                    version = processor[tool][:6]
                    d['tool_version'].append(f"{tool} {version}")
                elif tool == 'Go cmd/compile':
                    version = processor[tool][6:18]
                    tool = 'Go'
                    
                    d['tool_version'].append(f"{tool} {version}")
                elif tool == 'Apple LLVM':
                    
                    version = processor[tool][:8]
                    d['tool_version'].append(f"clang {version}")
                else:
                    d['tool_version'].append(f"{tool} -")
                    
                
                
            else:
                d['producer'].append(None)
                d['tool'].append(None)
                d['tool_version'].append(None)
                
            d['wasm'].append(k)
            
            
d = pd.DataFrame(d)
# display(d)

# display(d.groupby('language').count().reset_index())

# 1723 programs....maybe not ebough for a correct inference ?

# Now do some parsing of the binaries and gather some structural information ?
# Lets start with a simple profiling gathering
import subprocess
import os
import re

def nodize(t, LN):
    return [x for x in t.split(' ') if x]

def parse_s(start, content):
    # read until  
    buff = ''
    nodes = []
    operators = []

    i = start
    while i != len(content):
        c = content[i]
        if c == '"':
            #escape
            i = i + 1
            sc = content[i]
            escape = ""

            while sc != '"' and i < len(content) - 1:
                escape += sc
                i = i + 1
                sc = content[i]
            # split escape by zero ending bytes
            escape = escape.split('\\00')
            escape = [e for e in escape if e]
            # print(escape)

            i = i + 1
            #if escape:
                #print(escape)
            nodes += escape #nodize(escape, (-1, -1))
        elif c == '(':
            nnodes, newpos = parse_s(i + 1, content)
            i = newpos 
            if nnodes:
                # set previous buffer as a node

                if buff:
                    bh = buff
                    if "module" in bh:
                        bh = "module"

                    if bh.strip():
                        nodes += nodize(bh.strip(), (-1, -1))
                buff = ''
                nodes += [nnodes]
        elif c == ')':
            if buff:
                bh = buff
                if "module" in bh:
                    bh = "module"
                if bh.strip():
                    nodes += nodize(bh.strip(), (-1, -1))
            return nodes, i + 1
        else:
            buff += c
            i += 1
        
    return nodes[0], i

def pprint(tree, indent=0):
    operator, children = tree[0], tree[1:]

    tokens = [operator]
    #print("\t"*indent, operator)
    if type(children) == list:
        for ch in children:
            tokens += pprint(ch, indent + 1)
    return tokens
        
def clean_module(content):
    # Remove label annotations
    content, _ = re.subn(r'label (.*?)\n', '\n', content)
    # Remove inner comments for breaks ;@;;
    content, _ = re.subn(r';(.*?);', '', content)


    # Remove naming
    content, _ = re.subn(r'param ([0-9a-zA-Z\$\._-]+) i32', 'param_i32', content)
    content, _ = re.subn(r'param ([0-9a-zA-Z\$\._-]+) i64', 'param_i64', content)
    content, _ = re.subn(r'param ([0-9a-zA-Z\$\._-]+) f32', 'param_f32', content)
    content, _ = re.subn(r'param ([0-9a-zA-Z\$\._-]+) f64', 'param_f64', content)


    content, _ = re.subn(r'local ([0-9a-zA-Z\$\._-]+) i32', 'local_i32', content)
    content, _ = re.subn(r'local ([0-9a-zA-Z\$\._-]+) i64', 'local_i64', content)
    content, _ = re.subn(r'local ([0-9a-zA-Z\$\._-]+) f32', 'local_f32', content)
    content, _ = re.subn(r'local ([0-9a-zA-Z\$\._-]+) f64', 'local_f64', content)

    content, _ = re.subn(r'[ ]+i32', '', content)
    content, _ = re.subn(r'[ ]+f32', '', content)
    content, _ = re.subn(r'[ ]+i64', '', content)
    content, _ = re.subn(r'[ ]+f64', '', content)
    
    
    content, _ = re.subn(r'i32\.const \d+', 'i32.const', content)
    content, _ = re.subn(r'f32\.const \d+', 'f32.const', content)
    content, _ = re.subn(r'i64\.const \d+', 'i64.const', content)
    content, _ = re.subn(r'f64\.const \d+', 'f64.const', content)
    
    # Rename function names

    def replace(tk):
        #sys.stderr.write(f"{tk}")
        return "func "

    content, o = re.subn(r'func \$(.*?) ', replace, content)

    # Parse signature


    # replace call $x by call <>
    content, _ = re.subn(r'call ([0-9a-zA-Z\$\._]+)', 'call', content)
    # $env.abortOnCannotGrowMemory

    # replace local.get $x by local.get <> and local.set $x by local.set <>
    content, _ = re.subn(r'local.get ([0-9a-zA-Z\$\._-]+)', 'local.get', content)
    content, _ = re.subn(r'local.set ([0-9a-zA-Z\$\._-]+)', 'local.set', content)
    content, _ = re.subn(r'local.tee ([0-9a-zA-Z\$\._-]+)', 'local.tee', content)

    # replace global.get $x by global.get <> and global.set $x by global.set <>
    content, _ = re.subn(r'global.get ([0-9a-zA-Z\$\._-]+)', 'global.get', content)
    content, _ = re.subn(r'global.set ([0-9a-zA-Z\$\._-]+)', 'global.set', content)

    return content

def get_profiling(wasm, check_binary_in="../RQ1/all-binaries-metadata/all"):
    folded = subprocess.check_output(
        [
            os.environ.get("WASM2WAT"),
            "-f",
            f"{check_binary_in}/{wasm}"
        ]
    )
    folded = folded.decode()
    # clean a little bit
    folded = clean_module(folded)
    
    #print(folded)
    # s-expression parsing here
    nodes, _ = parse_s(0, folded)
    tokens = pprint(nodes)
    
    d = {}
    for t in tokens:
        if len(t) > 1:
            try:
                if t not in d:
                    d[t] = [0]
                d[t][0] +=1
            except Exception as e:
                print(e)
                pass
        
    return pd.DataFrame(d)
    
# d1 = get_profiling("0aa5df2feb9f301c0d53be8e380b971499cc8c8e737966c515e5ae0e1c40f4a0.wasm")
# print(d1)
# exit(0)

# read all wasms as vectors
dall = pd.DataFrame()

## Producer consumer for a faster processing
import threading
import time

ll = threading.Lock()
ll2 = threading.Lock()

worklist = []
results = []
processed = 0

for i, w in enumerate(d['wasm'].values):
    worklist.append(w)

def produce(i):
    global results
    global worklist
    while True:
        with ll:
            if len(worklist) == 0:
                break
            curr = worklist.pop()
        print(f"worker {i} taking {curr}")
        
        try:
            start = time.time()
            d1 = get_profiling(f"{curr}.wasm")
            end = time.time()
            print("Result in ", end-start, "seconds")
            with ll2:
                results.append(d1)
        except Exception as e:
            print(e)
            pass

def consume(expected):
    global processed
    global results
    global dall
    while True:
        time.sleep(10)
        if processed == expected:
            break
        
        with ll2:
            if len(results) > 0:
                chunk = results
                results = []
            else:
                print("No results yet")
                continue
        for r in chunk:
            dall = pd.concat([dall, r], axis = 0)
            processed += 1
        dall.to_csv("dall.csv")

        print("Processed", processed)
            
producers = []
expected = len(worklist)
for i in range(16): #16 producers:
    th = threading.Thread(target=produce, args=(i,))
    th.start()
    producers.append(th)

# consumer
consumer = threading.Thread(target=consume, args=(expected,))
consumer.start()

for t in producers:
    t.join()
    
consumer.join()

dall.to_csv("dall.csv")
# display(dall)