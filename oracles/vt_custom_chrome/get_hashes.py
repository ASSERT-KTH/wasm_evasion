import sys
import os
import hashlib

if __name__ == "__main__":
    hashes = []
    max_hahes = 400
    I = 0
    for dirpath, dirname, files in os.walk(sys.argv[1]):
        for f in files:
            if f.endswith(".wasm"):
                hsh = hashlib.sha256(open(os.path.join(dirpath, f), 'rb').read()).hexdigest()
                hashes.append(hsh)

                if len(hashes) >=  max_hahes:
                    I += 1
                    # print file out
                    tout = open("templates/template.yml", 'r').read()
                    content = ""
                    out = open(f"templates/job_{I}.yml", "w")
                    for h in hashes:
                        content += f'        - {{ hash: "{h}" }}\n'
                    out.write(tout.replace("%% HASHES", content))
                    out.close()
                    exit(0)

    I += 1
    # print file out
    tout = open("templates/template.yml", 'r').read()
    content = ""
    out = open(f"templates/job_{I}.yml", "w")
    for h in hashes:
        content += f'        - {{ hash: "{h}" }}\n'
    out.write(tout.replace("%% HASHES", content))
    out.close()