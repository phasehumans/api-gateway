interface User {
    name : string,
    age : number,
    email : string
}

function filterUsers (citizen : User[]){
    let ans = []

    for(let i= 0; i < citizen.length; i++){
        if(citizen[i]?.age > 18){
            ans.push(users[i])
        }
    }

    return ans
}