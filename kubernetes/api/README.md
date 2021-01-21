# ConsenSource API

## Deployment Steps

To deploy the API locally, we need to create the Kubernetes deployment and service resources.

Run the following command in the `api/` directory.

```
kubectl apply -f deployment.yaml -f service.yaml -f block-stream.yaml
```

### Create Ingress and Ingress Controller

We will deploy a Kubernetes ingress resource to `curl` the API via `localhost`. See the [ingress resource docs page](https://kubernetes.io/docs/concepts/services-networking/ingress/) for more info.

But before we can have a working ingress resource, we first need to install and deploy an [ingress-controller](https://kubernetes.io/docs/concepts/services-networking/ingress-controllers/). Specifically, we are using the [nginx ingress controller](https://kubernetes.github.io/ingress-nginx/deploy/#docker-for-mac) for Docker on Mac.

**Deploy Ingress Controller**

Run the following in the `api/` directory.

```
$ kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/controller-v0.40.2/deploy/static/provider/cloud/deploy.yaml --force
$ kubectl delete -A ValidatingWebhookConfiguration ingress-nginx-admission
```

**Deploy Ingress**

Run the following in the `api/` directory.

```
kubectl apply -f ingress.yaml
```

### Test Deployment

Now we should be able to ping the API from `localhost`. Run the following `curl` command:

```
$ curl http://localhost/api/health
```

The above command should return `OK`.
