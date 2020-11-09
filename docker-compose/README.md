# ConsenSource docker-compose

This application runs using separate Docker containers for the various components. These Docker images may be run together using the `docker-compose.yaml` file included within the repository.

## Setup 🔧

### Repo structure with Git Submodules

Each of the services required to run ConsenSource lives in it's own repo with it's own Dockerfile. In order to simplify the development process, our _docker-compose_ pattern takes an opinionated approach and assumes that the developer is using Git submodules to create a mono-repo like structure.

To get started with submodules, clone this repo with the following command:

```
$ git clone --recurse-submodules https://github.com/target/consensource.git
```

If you have already cloned but want submodules, run:

```
$ git submodule update --init --recursive
```

### Working with Git Submodules

#### Manual Update of a Single Submodule

To update a single submodule to the latest commit in master manually, `cd subdirectory` and run normal commands such as `git fetch`, `git pull`, and `git merge origin/master`. If you commit this after updating, the submodule
will be tied to the newest commit in the remote, and will update for others when they `pull`.

#### Less Manual Update

Run `git submodule update --remote $submodule_name`. This fetches and updates.

Even more fun, run

```
git submodule foreach git pull
```

to update all submodules.

Note: The command after `foreach` can be any arbitrary shell command.

For better logs when `diff`ing, add a config: `git config --global diff.submodule log`.

[Reference](https://git-scm.com/docs/git-submodule) |
[Git Book](https://git-scm.com/book/en/v2/Git-Tools-Submodules)

## Running ConsenSource 🚀

### With Docker Hub images

To start the ConsenSource application using the latest images from Docker Hub, run the following command in the project's root directory:

```
docker-compose up
```

You may need to first run the `down -v` command if the `up` command was run previously:

```
docker-compose down -v
```

You can also run the following command before running `up` to clear any caches:

```
docker system prune -f
```

### With locally built images

The main benefit of running local images is for local development. Changes can be made to the submodule code bases, built locally, and then run locally.

#### Build local images

Before running `docker-compose.local.yaml`, first build the component images:

```
docker-compose -f docker-compose.api.yaml build api
docker-compose -f docker-compose.cli.yaml build cli
docker-compose -f docker-compose.processor.yaml build processor
docker-compose -f docker-compose.sds.yaml build sds
```

You should now have the following images:

```
target/consensource-api:local
target/consensource-cli:local
target/consensource-processor:local
target/consensource-sds:local
```

#### Run local images

If you would like to run an image that you have built locally, run the following command in the project's root directory:

```
docker-compose -f docker-compose.local.yaml up
```

You may need to first run the `down -v` command if the `up` command was run previously:

```
docker-compose -f docker-compose.local.yaml down -v
```

You can also run the following command before running `up` to clear any caches:

```
docker system prune -f
```