// -h should give add, display, delete todo helper commands

const { Command } = require("commander");
const program = new Command();

let todos= []

program
  .name("todos")
  .description("CLI to some JavaScript string utilities")
  .version("0.8.0");

program
  .command("add")
  .description("add task")
  .argument("<string>")
//   .option("--first", "display just the first substring")
//   .option("-s, --separator <char>", "separator character", ",")
  .action((todo) => {
    // const limit = options.first ? 1 : undefined;
    todos.push(todo)
    // console.log(str.split(options.separator, limit));
    console.log(todos)
  });

program.parse();