#!/usr/bin/env node

/**
 * A little javascript sandbox to help me check somethings ...
 */

const path = require('node:path');
const fs = require('node:fs/promises')
const zlib = require('node:zlib');

const [, , ...args] = process.argv;

main(args);

function resolvePath(currentPath, basePath = process.cwd()) {
  if (path.isAbsolute(currentPath)) {
    return currentPath;
  }
  return path.resolve(basePath, currentPath);
}

/**
 * From https://github.com/isomorphic-git/isomorphic-git/blob/main/src/models/GitTree.js
 * @param {Buffer} buffer
 * @returns
 */
function parseBuffer(buffer) {
  const _entries = []
  let cursor = 0
  while (cursor < buffer.length) {
    const space = buffer.indexOf(32, cursor)
    if (space === -1) {
      throw new Error(
        `GitTree: Error parsing buffer at byte location ${cursor}: Could not find the next space character.`
      )
    }
    const nullchar = buffer.indexOf(0, cursor)
    if (nullchar === -1) {
      throw new Error(
        `GitTree: Error parsing buffer at byte location ${cursor}: Could not find the next null character.`
      )
    }
    let mode = buffer.slice(cursor, space).toString('utf8')
    if (mode === '40000') mode = '040000' // makes it line up neater in printed output
    const type = mode // mode2type(mode)
    const path = buffer.slice(space + 1, nullchar).toString('utf8')

    // Prevent malicious git repos from writing to "..\foo" on clone etc
    if (path.includes('\\') || path.includes('/')) {
      throw new UnsafeFilepathError(path)
    }

    const oid = buffer.slice(nullchar + 1, nullchar + 21).toString('hex')
    cursor = nullchar + 21
    _entries.push({ mode, path, oid, type })
  }
  return _entries
}

async function main(args) {
  switch (args[0]) {
    case 'decode-object': {
      const [, objectPath] = args;
      const blobContent = await fs.readFile(resolvePath(objectPath));
      zlib.unzip(blobContent, (err, buffer) => {
        if (err) {
          console.error(err);
          process.exit(1)
        }
        console.log(buffer.join(','))
        const blobString = buffer.toString()
        const [blobInfos, ...blobContent] = blobString.split('\x00');
        const [blobType, blobLength] = blobInfos.split(' ');
        console.log('blobType', blobType, 'blobLength', blobLength)
        if (blobType !== 'tree') {
          console.log(blobContent.join(''));
        }
        else {
          console.log(parseBuffer(buffer));
        }
      })
      break;
    }
    default:
      console.log(`
  Basic debugging sandbox in nodejs
      `);
  }
}
