function getMax (nums: number[]){
    let maxi = -1000000

    for(let i = 0; i < nums.length; i++){
        if (nums[i] > maxi){
            maxi = nums[i]
        }
    }

    return maxi
}


const res = getMax([1,2,3,4])


// multi address - > adress[]