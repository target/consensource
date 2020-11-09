# ConsenSource Database

## Deployment Steps

### Postgres Deployment

**Create Secrets**

First, create the secret resources containing the database credentials.

Run the following command in the `postgres/` directory.

```
kubectl apply -f secrets.yaml
```

**Deploy Resources**

To run our local Postgres database service, we need to create the Kubernetes deployment, service, persistent volume, and persistent volume claim resources.

Run the following command in the `postgres/` directory.

```
kubectl apply -f deployment.yaml service.yaml storage.yaml
```

The `PersistentVolume` and `PersistentVolumeClaim` resources allow us to partition some space from local disk for storing data.

### Setup Database Tables

Enter the pod and run the SQL script to create the tables, roles, etc.

```
kubectl exec -it postgres-db-hash_of_pod bash
$ psql --username=consensourcedb -f /docker-entrypoint-initdb.d/consensource_tables.sql
```

Now, run the following to see if the relations (i.e. tables) have been created:

```
$ psql --username=consensourcedb
$ consensourcedb=# \d
```

If other ConsenSource components have been deployed, such as the `validator` and the `sds`, then you can query the `blocks` table to see the genesis block record.

```
$ consensourcedb=# select * from blocks;
```

## Tear-down Instructions

### Clear Data In Tables

Sometimes you'll need to clear all the data in the tables when testing changes.

Run the following command to clear all the data in the consensource tables:

```
kubectl exec -it postgres-db-hash_of_pod bash
$ psql --username=consensourcedb -f scripts/clear_tables.sql
```

### Delete Tables, Roles, Indexes, etc.

When changes are made to the database relations (e.g. adding/removing table rows, updating rows, adding roles, etc.), it's usually easier to run the tear down script and rerun the tables script.

Run the following command to completely teardown ConsenSource.

```
kubectl exec -it postgres-db-hash_of_pod bash
$ psql --username=consensourcedb -f scripts/clear_tables.sql
$ psql --username=consensourcedb -f scripts/teardown.sql
```