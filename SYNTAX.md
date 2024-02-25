## 语法

- [语法](#语法)
    - [注释](#注释)
    - [let](#let)
    - [static](#static)
    - [const](#const)
    - [宏](#宏)
    - [格式化输出](#格式化输出)
    - [fn](#fn)
    - [if](#if)
    - [match](#match)
    - [while](#while)
    - [\>\<（交换）](#交换)

#### 注释

```
// 双斜杠注释
/*
    块注释
*/
```

#### let

你需要用let在运行时绑定一个变量

```
let a = 10;
```

#### static

定义静态变量

```
static STATIC_VAR = 114514 / 5;
export static A = 1919810;
```

带有`export`关键字的静态变量可以在游戏中访问到。
下面的命令可以在游戏中通过其他方式将A变量设置为123456。

```
/scoreboard players set MCSH A 123456
```

#### const

你可以利用`const`绑定常量。注意，**只有常量可以为字符串**。

```
const FOO = 10 + 20;
const BAR = "可以用中文";
```

#### 宏

调用宏的格式是：名称 + `!` + 圆括号或方括号或花括号 + 符合宏自定义语法的内容。
目前还不能自定义宏。

例如，print宏的语法是`print!(选择器, 格式化文本)`，则您可以这样：

```
print![@a[r = 10], "{#bold}Hello, I am {@s}, you are near to me!"];
```

很显然这并不符合函数调用的语法，但是它可以在print宏内正常使用。
因此，宏可以自定义任何内容，只要不 使用违规符号/括号不匹配/双引号不匹配 即可。
即您在满足词法正常分析的基础上，可以自定义任意语法。

#### 格式化输出

`{var}`将打印定义的变量、静态变量或常量，`{#style}`将使用样式。可使用的样式在[README.md](README.md#语法)中可以找到。

```
const HELLO_STR = "Hello";
let a = 10;

// 打印一个前面红色、后面红色粗体的“Hello, you have 10 coins!"在聊天框
print!(@a[tag=human, r=20], "{#red}{HELLO_STR}, {#bold}you have {a} coins!");

title!(@s[dz=10], "{#}")
```

#### fn

定义一个返回`a + b`的函数
```
fn foo(a, b) {
    return a + b;
}
```

定义一个导出函数。注意，导出函数不允许携带参数，否则会导致编译报错。

```
export fn bar(){
    let a = 12;
    let b = 10 + 18;
    let c = foo(a, b);
    print!(@a, "c = {c}");
}
```

调用函数

```
let c = foo(1, 2); // c = 3
```

#### if

```
if a > 10 {
    // do some
} else if a == 0 {
    // do some
} else {
    // do some
}
```

#### match

匹配任意分支或默认分支。默认分支模式为`..`。
匹配语法会采用二分法匹配，所以可以有大量匹配模式。
```
match a {
    1 => {
        print!(@a, "a = 1");
    },

    // 顺序是无所谓的
    100 => {
        print!(@a, "a = 100");
    },

    2 => {
        print!(@a, "a = 2");
    },

    .. => {
        print!(@a, "a没有在预期值中");
    } // 注意末尾不能多逗号
}
```

#### while
```
let a = 5;
while a > 0 {
    print!(@a, "a = {a}");
    a = a - 1;
}
```
将打印
```
a = 5
a = 4
a = 3
a = 2
a = 1
```

#### ><（交换）

`><`语法可以交换两个变量的值。如果两端不是变量则没有意义，将发生编译时错误。

```
let a = 10;
let b = 2;
a >< b;

// a = 2, b = 10
print!(@a, "a = {a}, b = {b}");
```