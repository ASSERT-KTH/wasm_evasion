import json

def get_node_sets(nodes, total, operator="+"):
        
        
    def merge(sets):
        #print(sets)
        r = sets[0]
        
        if operator == "+":
            op = r.union
        elif operator == "i":
            op = r.intersection
        
        for c in sets[1:]:
            r = op(c)
            
            if operator == "+":
                op = r.union
            elif operator == "i":
                op = r.intersection
        
        return r
    
    def get_combinations(sets, size=2, plot_percentage=True):
        #names, sets = [s[0] for s in sets], [s[1] for s in sets]
        combs = [c for c in itertools.combinations(sets, size)]
        
        r = []
        
        edges = []
        for c in combs:
            #print("Combination", c)
            names, sets = [ cc[0] for cc in c ], [cc[1] for cc in c]
            merged = merge(sets)
            
            #print(names)
            id = f"-{operator}-".join(names)
            s = (len(merged)/total) if plot_percentage else len(merged)
            #id = f"{id}({s})"
            print(id)
            
            for i in sorted(names):
                if id != f"{i}":
                    
                    color='black'

                    if len(merged) == 0:
                        color='C1'
                    edges.append((id, f"{i}", dict(color='black')))
                    
            color='C0'
            
            if size==1:
                color='C2'
            elif len(merged) == 0:
                color='C1'
            #print(len(merged), s)
            label = id.replace(f"-{operator}-", "\n")
                
            r.append((id, dict(count=len(merged), size=len(merged), set=merged, id=id, label=label, s=s, color=color)))
            
        return r, edges
    
    allnodes = []
    alledges = []
    for k in range(1, len(nodes) + 1):
        newsets, edges = get_combinations(nodes, k)
        
        allnodes +=  newsets
        alledges += edges
        
    return allnodes, alledges


def load_sets(name, find_mutation_info=False):
    a = open(name, 'r').read()
    data = json.loads(a)
    
    sets = {}
    
    upto=-1
        
    for i, t in enumerate(data[:upto]):
        name = t['id']
        mutations = t['mutations']
        
        
        for m in mutations:            
            if find_mutation_info:
                try:
                    r = []
                    count, file = m['map']
                    # load file
                    f = open(file, "r").read()
                    f = json.loads(f)
                    m['map'] = f
                        
                    if ('Peephole' in m['class_name'] or 'Codemotion' in m['class_name']) and len(m['map']) == 0:
                        continue
                        
                except Exception as e:
                    print(e)
                    break
            
            if m['class_name'] not in sets:
                sets[m['class_name']] = []
            sets[m['class_name']].append(t)

   
    for k, v in sets.items():
        print(k, len(v))
    return list(zip(sets.keys(), sets.values())), len(data[:upto]), data[:upto]

