/* 
- node binds code with wrapper fn -> execute() and parse to v8
- execute(exports, require, module, __filename, __dirname) {// actual code} <--- executeWrapper fn
    - require  fn: to import module, w/ module id as args, start w/ . then check in exports {}, or w/o . then checks in node built-in module
    - dir, filename: to know self location
    - exports fn: require brings fn that are in exports {}
    - module fn: default export; module.exports= function (){// body}
- per file one default export (module.exports) & multi exports
- wrapper fn args: exports, require, module, __filename, __dirname

*/