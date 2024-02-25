## max(a, b)
返回a和b中的最大值

```
let a = max(2, 1); // a = 2
let b = max(5, 20); // b = 20
```

## min(a, b)
返回a和b中的最小值。和`max`函数类型

## random(min, max)
将从min到max范围中取随机值。**min和max必须为常量**。

```
// a将会被设定为10~15内的随机值，包括10和15。
let a = random(10, 10 + 5);
```

如果您希望该区间在运行时确定，则可以采用以下方法：

```
fn my_random(min, max) {
    const INT_MAX = 2147483467;
    const INT_MIN = -2147483468;

    if min >= max {
        print!(@s, "{#red}随机数min必须小于max");
        return; // 返回0
    }
    let division = (INT_MAX - INT_MIN) / (max - min);
    let rand = random(INT_MIN, INT_MAX);
    return min + rand / division;
}

export static INPUT = 0;
export main() {
    let c = my_random(INPUT, INPUT + 20);
    print!(@a, "the random number is {c}");
}
```

## print!

`print!(选择器, 格式化文本)`

例如

```
print![@a[r = 10], "{#bold}Hello, I am {@s}, you are near to me!"];
```
可能会以粗体打印下面内容
```
Hello, I am Steve, you are near to me!
```

## title!

`title!(选择器，title|subtitle|actionbar, 格式化文本)`

例如

```
title!(@a, title, "HELLO!");
```

会在所有人的屏幕上显示大大的“HELLO”

## run!

`run! [指令字符串, 指令字符串, ...]`

例如

```
run![
    "say aaaa",
    "say bbbb",
    "tag @s add printed"
];
```

将会以命令执行者的身份说出“aaaa”和“bbbb”后给自己添加“printed”标签。

## run_concat!

`run_concat!(参数, 参数, 参数, ...)`

例如

```
const THING = "stone";
const COUNT = 2;

run_concat!("give @s ", THING, " ", COUNT, " 0");
```

将给自己两个石头。
