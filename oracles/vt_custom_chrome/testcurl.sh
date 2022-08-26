curl -X POST --user 'vt:vt123' -F "file=@test.wasm" http://localhost:4000/upload_file
curl --user 'vt:vt123' http://localhost:4000/get_all_results
