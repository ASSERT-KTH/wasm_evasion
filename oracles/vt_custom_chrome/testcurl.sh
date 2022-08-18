curl -X POST --user 'admin:admin' -F "file=@test.wasm" http://localhost:5000/upload_file
curl --user 'admin:admin' http://localhost:5000/get_all_results
