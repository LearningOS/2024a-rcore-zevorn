# 简答作业

# 第 1 题

**Question**：正确进入 U 态后，程序的特征还应有：使用 S 态特权指令，访问 S 态寄存器后会报错。 请同学们可以自行测试这些内容（运行 三个 bad 测例 (ch2b_bad_*.rs) ）， 描述程序出错行为，同时注意注明你使用的 sbi 及其版本。

**Answer**:

报错输出：

```bash
[kernel] Loading app_0
[kernel] PageFault in application, kernel killed it.
[kernel] Loading app_1
[kernel] IllegalInstruction in application, kernel killed it.
[kernel] Loading app_2
[kernel] IllegalInstruction in application, kernel killed it.
```

app_0 对应 `ch2b_bad_address`，用户态代码中访问了非法地址 0 :

```rust
    #[allow(clippy::zero_ptr)]
    (0x0 as *mut u8).write_volatile(0);
```

此时会触发 cpu 异常，进入异常处理函数:

```rust
pub fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
    let scause = scause::read(); // get trap cause
    let stval = stval::read(); // get extra value
                               // trace!("into {:?}", scause.cause());
    match scause.cause() {
        ...
        Trap::Exception(Exception::StoreFault) | Trap::Exception(Exception::StorePageFault) => {
            println!("[kernel] PageFault in application, bad addr = {:#x}, bad instruction = {:#x}, kernel killed it.", stval, cx.sepc);
            exit_current_and_run_next();
        }
        ...
    }
}
```

app_1 在用户态执行了一个特权指令 `sret`:

```rust
    unsafe {
        core::arch::asm!("sret");
    }
```

这是无效指令，所以进入 trap_handler 以后，匹配到:

```rust
pub fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
    let scause = scause::read(); // get trap cause
    let stval = stval::read(); // get extra value
                               // trace!("into {:?}", scause.cause());
    match scause.cause() {
        ...
        Trap::Exception(Exception::IllegalInstruction) => {
            println!("[kernel] IllegalInstruction in application, kernel killed it.");
            exit_current_and_run_next();
        }
        ...
    }
    cx
}
```

app_2 与 app_1 类似, 只不过是在用户态访问了 CSR 寄存器，CSR 寄存器只能在 S 态及更高特权态中才能访问，所以会触发异常，进入异常处理函数。

```rust
    unsafe {
        core::arch::asm!("csrr {}, sstatus", out(reg) sstatus);
    }
```

# 第 2 题

深入理解 trap.S 中两个函数 __alltraps 和 __restore 的作用，并回答如下问题:

## 2.1

**Question**：L40：刚进入 __restore 时，a0 代表了什么值。请指出 __restore 的两种使用情景。

**Answer**:

1. 刚进入 __restore 时，a0 代表了当前内核态的栈顶；
2. __restore 的两种使用情景：任务切换，系统调用回到用户态。

## 2.2

**Question**：L43-L48：这几行汇编代码特殊处理了哪些寄存器？这些寄存器的的值对于进入用户态有何意义？请分别解释。

```
ld t0, 32*8(sp)
ld t1, 33*8(sp)
ld t2, 2*8(sp)
csrw sstatus, t0
csrw sepc, t1
csrw sscratch, t2
```

**Answer**:

L43：读取 sstatus 寄存器的值；
L44：读取 sepc 寄存器的值，当 Trap 是一个异常的时候，sepc 记录 Trap 发生之前执行的最后一条指令的地址；
L45：读取 sscratch 寄存器的值。
L46：将 t0 值写入 sstatus 寄存器；
L47：将 t1 值写入 sepc 寄存器；
L48：将 t2 值写入 sscratch 寄存器。

这些值用于恢复到用户态任务的返回点。

## 2.3

**Question**：L50-L56：为何跳过了 x2 和 x4？

```
ld x1, 1*8(sp)
ld x3, 3*8(sp)
.set n, 5
.rept 27
   LOAD_GP %n
   .set n, n+1
.endr
```

**Answer**:

x2 指向的是内核栈，后面还会用到；x4 一般不会使用到，所以不需要备份。

## 2.4

**Question**： L60：该指令之后，sp 和 sscratch 中的值分别有什么意义？

```
csrrw sp, sscratch, sp
```

**Answer**:

`csrrw rd, csr, rs` 作用是将特定的 csr 寄存器的值读取到 rd 中，再将 rs 的值写入 csr 寄存器。

因此这条指令在这里是对 sp 和 sscratch 进行了一个交换，

该指令执行之后， sp 重新指向用户栈栈顶，sscratch 也依然保存 进入 Trap 之前的状态并指向内核栈栈顶。

## 2.5

**Question**：__restore：中发生状态切换在哪一条指令？为何该指令执行之后会进入用户态？

**Answer**:

`sret` 指令，其作用是：是从 S 特权模式返回陷入到 S 特权模式之前的特权模式继续执行。

这里都是从用户态通过 trap 陷入的 S 特权模式，然后通过 `sret` 返回到用户态。

## 2.6

**Question**：L13：该指令之后，sp 和 sscratch 中的值分别有什么意义？

```
csrrw sp, sscratch, sp
```

**Answer**:

这条指令执行完以后 sp 指向内核栈， sscratch 指向用户栈。

## 2.7

**Question**：从 U 态进入 S 态是哪一条指令发生的？


**Answer**:

`ecall` 指令。在 RISC-V 架构中，ecall 指令用于触发系统调用或异常处理。当用户态程序需要请求内核服务时，会执行 ecall 指令。这会导致处理器进入一个更高特权级别的异常处理程序，在那里操作系统可以检查调用者提供的参数并执行相应的系统调用服务。

----

# 荣誉准则

在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

1. 与助教卢荣昌交流了我在 `os/src/task/mod.rs` 的 `TaskManager` 结构体中实现了第一版系统调用计数字段方面的实现思路；
2. 后续调试发现，需要为每一个 task 进行单独记录系统调用次数。

此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

无

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。

