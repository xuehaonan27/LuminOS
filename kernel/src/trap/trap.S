.attribute arch, "rv64gc" # Make LLVM happy

.set REGISTER_SIZE, 8 # On 64-bit machine, should be 4 one 32-bit machine
.set F_REGISTER_SIZE, 8 # On D Extension RISCV machine

.altmacro
.macro SAVE_GP n
    sd x\n, \n*REGISTER_SIZE(sp)
.endm
.macro LOAD_GP n
    ld x\n, \n*REGISTER_SIZE(sp)
.endm

    .section .text.trampoline
    .globl __alltraps
    .globl __restore
    .align 2
__alltraps:
    csrrw sp, sscratch, sp
    # now sp->*TrapContext in user space, sscratch->user stack
    # save other general purpose registers
    sd x1, 1*REGISTER_SIZE(sp)
    # skip sp(x2), we will save it later
    sd x3, 3*REGISTER_SIZE(sp)
    # skip tp(x4), application does not use it
    # save x5~x31
    .set n, 5
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr
    # we can use t0/t1/t2 freely, because they have been saved in TrapContext
    csrr t0, sstatus
    csrr t1, sepc
    sd t0, 32*REGISTER_SIZE(sp)
    sd t1, 33*REGISTER_SIZE(sp)
    # read user stack from sscratch and save it in TrapContext
    csrr t2, sscratch
    sd t2, 2*REGISTER_SIZE(sp)
    # load kernel_satp into t0
    ld t0, 34*REGISTER_SIZE(sp)
    # load trap_handler into t1
    ld t1, 36*REGISTER_SIZE(sp)
    # move to kernel_sp
    ld sp, 35*REGISTER_SIZE(sp)
    # switch to kernel space
    csrw satp, t0
    sfence.vma
    # jump to trap_handler
    jr t1

__restore:
    # a0: *TrapContext in user space(Constant); a1: user space token
    # switch to user space
    csrw satp, a1
    sfence.vma
    csrw sscratch, a0
    mv sp, a0
    # now sp points to TrapContext in user space, start restoring based on it
    # restore sstatus/sepc
    ld t0, 32*REGISTER_SIZE(sp)
    ld t1, 33*REGISTER_SIZE(sp)
    csrw sstatus, t0
    csrw sepc, t1
    # restore general-purpuse registers except sp/tp
    ld x1, 1*REGISTER_SIZE(sp)
    ld x3, 3*REGISTER_SIZE(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    # back to user stack
    ld sp, 2*REGISTER_SIZE(sp)
    sret
