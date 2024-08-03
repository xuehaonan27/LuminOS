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

    .section .text
    .globl __alltraps
    .globl __restore
    .align 2
__alltraps:
    # check user / kernel mode
    csrr tp, sstatus
    andi tp, tp, 0x100 # sstatus & (1 << 8); check SPP
    beqz tp, __user_trap_start # SPP = 0, user mode trap
    j __real_trap_entry # SPP = 1, kernel mode trap, do not modify stack pointer
__user_trap_start:
    csrrw sp, sscratch, sp
__real_trap_entry:
    # now sp->kernel stack, sscratch->user stack
    # allocate a TrapContext on kernel stack
    addi sp, sp, -34*REGISTER_SIZE # general purpose registers and 2 csr registers
    # save general-purpose registers
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
    # we can use t0/t1/t2 freely, because they were saved on kernel stack
    csrr t0, sstatus
    csrr t1, sepc
    sd t0, 32*REGISTER_SIZE(sp)
    sd t1, 33*REGISTER_SIZE(sp)
    # read user stack from sscratch and save it on the kernel stack
    csrr t2, sscratch
    sd t2, 2*REGISTER_SIZE(sp)
    # set input argument of trap_handler(cx: &mut TrapContext)
    mv a0, sp
    call trap_handler

__restore:
    # now sp->kernel stack(after allocated), sscratch->user stack
    # restore sstatus/sepc
    ld t0, 32*REGISTER_SIZE(sp)
    ld t1, 33*REGISTER_SIZE(sp)
    ld t2, 2*REGISTER_SIZE(sp)
    csrw sstatus, t0
    csrw sepc, t1
    csrw sscratch, t2

    # check SPP
    andi t0, t0, 0x100
    bnez t0, __kernel_trap_end # SPP != 0, kernel mode trap

__user_trap_end:
    # restore general-purpuse registers except sp/tp
    ld x1, 1*REGISTER_SIZE(sp)
    ld x3, 3*REGISTER_SIZE(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    # release TrapContext on kernel stack
    addi sp, sp, 34*REGISTER_SIZE
    # now sp->kernel stack, sscratch->user stack
    csrrw sp, sscratch, sp
    sret

__kernel_trap_end:
    # restore general-purpuse registers except sp/tp
    ld x1, 1*REGISTER_SIZE(sp)
    ld x3, 3*REGISTER_SIZE(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    # release TrapContext on kernel stack
    addi sp, sp, 34*REGISTER_SIZE
    # do not exchange sp and sscratch
    sret