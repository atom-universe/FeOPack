import title, { num, plusNum } from "./app.js";

title("Hello FeOPack");

// 这里我突然意识到，为什么之前说 import {} 不能看作是解构
// 比如这个 num，引入的还是一个引用，如果解构成了，那怎么还能 plusNum 改变 num 的值？
console.log(num);
plusNum();
console.log(num);

