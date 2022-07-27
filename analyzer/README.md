## Tests

- Create a mongodb instance:
```bash
docker run -it -e MONGO_INITDB_ROOT_USERNAME=admin -e MONGO_INITDB_ROOT_PASSWORD="admin" -v $(pwd)/mongodb:/data/db -p 27017:27017 --name mongodb2 -d mongo
```

