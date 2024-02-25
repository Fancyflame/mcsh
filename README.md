# MCSH

MCSH语言是一个语法类似Rust的编译型编程语言，其编译目标是mcfunction文件，以在Minecraft中运行。

MCSH有内存条，可实现函数递归操作。

- [MCSH](#mcsh)
  - [编译](#编译)
  - [使用](#使用)
  - [CLI](#cli)
      - [在虚拟仿真运行](#在虚拟仿真运行)
      - [编译](#编译-1)
  - [语法](#语法)
  - [标准库](#标准库)

## 编译

您需要先[安装Rust](https://www.rust-lang.org/zh-CN/learn/get-started)
然后在您的控制台运行

```shell
git clone https://github.com/Fancyflame/mcsh.git
cd mcsh
cargo r
```

在此期间，请保证您的网络通畅，因为rust编译需要通过网络下载许多必要的依赖。

## 使用

在编译完成后，将该行为包安装到Minecraft里。游戏中加载该行为包后，在**第一次使用前**必须先运行

```
/function mcsh_init
```

然后再调用导出的函数

```
/function print_some
/function your_custom_function
```

如果您运行`mcsh_init`后再次调用，则会将所有静态变量重置为初始值。您可以利用这一点来重置环境。

## CLI

MCSH编译器是一个命令行工具，您可以使用`./mcsh --help`查看帮助文档。在rust中，`cargo r`也是运行程序的命令（如果没有程序或代码变动则编译）。下面给出一些常用的示例命令。

**在MCSH CLI中，所有相对路径都被视为基于当前工作目录。** 你可以利用`--help`选项探索更多功能。

#### 在虚拟仿真运行

模拟运行本仓库里的print.mcsh示例文件的print_some函数。

```shell
cd mcsh
cargo r examples/print.mcsh simulate print_some
```

#### 编译

将本仓库里的print.mcsh示例文件编译到`C:\Users\Alice\Desktop\mcsh_out`目录下（没有生成额外文件夹，请保证该文件夹是空的！），并使用交互式输入（`-m`）生成`manifest.json`。

```shell
cd mcsh
cargo r examples/print.mcsh b -o "C:\Users\Alice\Desktop\mcsh_out" -m
```

输出文件夹结构如下所示。可以观察到，mcsh额外生成了一个`mcsh_init.mcfunction`文件用于初始化环境。

```
.
|-- functions
|   |-- MCSH
|   |   |-- __MCSH_Private_AnonymousLabel_0.mcfunction
|   |   |-- __MCSH_Private_AnonymousLabel_1.mcfunction
|   |   |-- __MCSH_Private_AnonymousLabel_2.mcfunction
|   |   |-- __MCSH_Private_AnonymousLabel_3.mcfunction
|   |   |-- __MCSH_Private_AnonymousLabel_4.mcfunction
|   |   |-- __MCSH_Private_AnonymousLabel_5.mcfunction
|   |   |-- __MCSH_Private_AnonymousLabel_6.mcfunction
|   |   |-- __MCSH_Private_AnonymousLabel_7.mcfunction
|   |   |-- __MCSH_Private_MemoryLoad_Chunks1
|   |   |   |-- Branch0_1.mcfunction
|   |   |   |-- Branch0_15.mcfunction
|   |   |   |-- Branch0_3.mcfunction
|   |   |   |
|   |   |   |   ... many files ...
|   |   |   |
|   |   |   |-- Leaf8.mcfunction
|   |   |   `-- Leaf9.mcfunction
|   |   |-- __MCSH_Private_MemoryLoad_Chunks1.mcfunction
|   |   |-- __MCSH_Private_MemoryStore_Chunks1
|   |   |   |-- Branch0_1.mcfunction
|   |   |   |-- Branch0_15.mcfunction
|   |   |   |-- Branch0_3.mcfunction
|   |   |   |
|   |   |   |   ... many files ...
|   |   |   |
|   |   |   |-- Leaf8.mcfunction
|   |   |   `-- Leaf9.mcfunction
|   |   `-- __MCSH_Private_MemoryStore_Chunks1.mcfunction
|   |-- mcsh_init.mcfunction
|   `-- print_some.mcfunction
`-- manifest.json
```

## 语法

转到[SYNTAX.md](SYNTAX.md)阅读语法

其中，格式化文本可使用的样式有
|标准名称            |别名        |
|-------------------|------------|
|black              |            |
|dark_blue          |            |
|dark_green         |            |
|dark_aqua          |            |
|dark_red           |            |
|dark_purple        |            |
|gold               |            |
|gray               |            |
|dark_gray          |            |
|blue               |            |
|green              |            |
|aqua               |            |
|red                |            |
|light_purple       |magenta     |
|yellow             |            |
|white              |            |
|minecoin_gold      |dark_yellow |
|material_quartz    |quartz      |
|material_iron      |iron        |
|material_netherite |netherite   |
|obfuscated         |rand_char   |
|bold               |            |
|material_redstone  |redstone    |
|material_copper    |copper      |
|italic             |            |
|material_gold      |dark_gold   |
|material_emerald   |emerald     |
|reset              |            |
|material_diamond   |diamond     |
|material_lapis     |laps        |
|material_amethyst  |amethyst    |

## 标准库

标准库请转到[STD.md](STD.md)。