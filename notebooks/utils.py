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