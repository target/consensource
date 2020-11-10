# ConsenSource Kubernetes

This repo provides instructions on how to deploy ConsenSource to a locally running Kubernetes cluster using Docker Desktop.

## Prerequisites

### Installation

1. [Docker Desktop](https://www.docker.com/products/docker-desktop) for Mac
2. Clone this repository

### Enable Kuberentes on Docker Desktop

1. Open up `Preferences...` in the Docker Desktop app
2. Under the `Kubernetes` tab click the `Enable Kubernetes` checkbox
3. Optionally, click the `Show system containers (advanced)` checkbox
4. Click the `Apply & Restart` button on the bottom right
5. Once Docker restarts, you should see `Kubernetes is running` under the Docker icon and the preferences UI
6. Run `kubectl config get-contexts` and you should now see the `docker-desktop` namespace.
7. Run `kubectl config use-context docker-desktop` to switch to the `docker-desktop` namespace.

## Deployment Steps

Run the following commands in the root directory of this repo.

### 1. Validator

First, deploy the validator:

```
kubetcl apply -f validator/
```

Before deploying other components, wait until the validator pod's status says `RUNNING`. To check pods' statuses, run:

```
kubectl get pods
```

### 2. Postgres Database

Second, deploy the Postgres database. Follow [this README's](postgres/README.md) `Deployment Steps` section for instructions.

### 3. SDS and Processor

Third, deploy the SDS and processor components.

```
kubetcl apply -f sds/
kubetcl apply -f processor/
```

### 4. API

Fourth, deploy the API. Follow [this README](api/README.md) for instructions.

### 5. UI

Lastly, deploy the UI. Follow [this README](ui/README.md) for instructions.

## Tear-down and Redeployment

If you want to test changes made to components, you may need tear down an existing deployment, redeploy all components, and wipe the database.

1. First, [clear the database](https://github.com/target/consensource/tree/master/kubernetes/postgres#clear-data-in-tables) or [tear down the tables](https://github.com/target/consensource/tree/master/kubernetes/postgres#delete-tables-roles-indexes-etc)

2. Second, run the following commands to tear down the deployments.

  ```
  kubectl delete deployment --all
  ```

3. (Optional) It's usually unnecessary to delete the service resources, but if necessary:

```
kubectl service --all
```

Go through the above steps in the Deployment section to redeploy and setup the database.