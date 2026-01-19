console.log("Hi");

const data= fetch('https://api.freeapi.app/api/v1/public/quotes?page=1&limit=10&query=human')

// console.log(data);
data.then(()=> console.log("Data aa gaya"))
data.catch(() => console.log("Data nahi aya"))
console.log("Byee");