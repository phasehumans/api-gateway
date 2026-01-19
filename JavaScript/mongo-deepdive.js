/* 
db : data is stored persistently
- db don't have granual access as 1st class citizen
- db allow acces to 4 primitives : CRUD

ORM:
- talk to db, ex. prisma
- create -> create
- read -> findbyid, find, findone
- update -> update, updateone $push
- delete -> deletemany, deleteone



cluster [db, db, db]
db  [table, table, table]
table
*/