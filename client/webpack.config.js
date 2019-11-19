var path = require('path')

module.exports = {
  entry: {
    auditor: './src/entrypoint_auditor.js',
    retailer: './src/entrypoint_retailer.js',
    factory: './src/entrypoint_factory.js',
    standards_body: './src/entrypoint_standards_body.js'
  },
  output: {
    filename: '[name].js',
    path: path.resolve(__dirname, 'public/js')
  },

  resolve: {
    alias: {
      App: path.resolve(__dirname, './src'),
      zeromq$: path.resolve(__dirname, './src/mock_zmq.js')
    }
  }
}
