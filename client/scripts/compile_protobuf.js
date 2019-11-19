'use strict'

const protobuf = require('protobufjs')

const path = require('path')
const fs = require('fs')

const proto_dir = 'protos'

let root = new protobuf.Root()

let files = fs.readdirSync(proto_dir)
  .map(f => path.resolve(proto_dir, f))
  .filter(f => f.endsWith('.proto'))

try {
  root = root.loadSync(files)
} catch (e) {
  console.log('Unable to load protobuf files!')
  throw e
}

let output = JSON.stringify(root, null, 2)

if (output !== '') {
  process.stdout.write(output, 'utf8')
}
