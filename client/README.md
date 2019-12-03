# Client

The ConsenSource UI is comprised of a number of Mithril applications under the same router. The default application that is served from the `index.html` page is for retailers. To access the other apps, specify the relative .html file in [/public](/client/public) such as `localhost:8080/index_auditor.html`.

## Env Setup

  - Node8: `brew install node@8`

## Building

  - `npm install`
  - `npm run build-dev`

## Development

### Running 
Running in watch mode is the easiest way to develop as it will compile changes on save. This can be started by running:

```
$ npm run watch

--- Note: This command will not compress your build - it is therefore recommended to perform final testing with a production build. ---
```

After building the client, _cd_ root of the repo repo and run:

```
$ docker-compose -f docker-compose.yaml up
```

This will start an httpd server, with the files hosted available at
[http://localhost:8080](http://localhost:8080).

### Linting

Code linting is provided through [ESLint](eslint.org) and is configured with the
`.eslint.js` file at the root of the `client` directory. It can be run via:

```
$ npm run lint
```

Formatting code can be done by running

```
$ npm run format
```

This will format the code using [es-beautifier](https://github.com/dai-shi/es-beautifier),
which is the same tool in the standard VSCode plugin, among others.

A non-zero return status means your code is linty.
