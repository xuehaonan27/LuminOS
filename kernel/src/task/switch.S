.attribute arch, "rv64gc" # Make LLVM happy

.set REGISTER_SIZE, 8 # On 64-bit machine
.set F_REGISTER_SIZE, 8 # On D Extension RISCV machine

.altmacro
.macro SAVE_SN n
    sd s\n, (\n+2)*REGISTER_SIZE(a0)
.endm
.macro LOAD_SN n
    ld s\n, (\n+2)*REGISTER_SIZE(a1)
.endm
.macro SAVE_FSN n
    fsd fs\n, (14*REGISTER_SIZE+\n*F_REGISTER_SIZE)(a0)
.endm
.macro LOAD_FSN n
    fld fs\n, (14*REGISTER_SIZE+\n*F_REGISTER_SIZE)(a1)
.endm

# Only ra, sp, sx registers need to be saved by assembly code.
# Other registers' saving and restoring should be handled by rustc.

    .section .text
    .globl __switch
__switch:
    # Phase 1
    # __switch(
    #   current_task_cx_ptr: *mut TaskContext,
    #   next_task_cx_ptr: *const TaskContext,
    # )

    # Phase 2
    # save kernel stack of current task
    # a0 holds pointer to current TaskContext
    # save kernel stack of current task
    sd sp, REGISTER_SIZE(a0)
    # save ra & s0 ~ s11 of current execution
    sd ra, 0(a0)
    .set n, 0
    .rept 12
        SAVE_SN %n
        .set n, n+1
    .endr
    # save fs0 ~ fs11 of current execution
    .set n, 0
    .rept 12
        SAVE_FSN %n
        .set n, n+1
    .endr

    # Phase 3
    # restore fs0 ~ fs11 of next execution
    .set n, 0
    .rept 12
        LOAD_FSN %n
        .set n, n+1
    .endr
    # restore ra & s0~s11 of next execution
    # a1 holds pointer to next TaskContext
    ld ra, 0(a1)
    .set n, 0
    .rept 12
        LOAD_SN %n
        .set n, n+1
    .endr
    # restore kernel stack of next task
    ld sp, REGISTER_SIZE(a1)

    # Phase 4
    ret