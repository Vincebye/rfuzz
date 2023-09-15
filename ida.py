import idc
import idaapi
import idautils
import ida_segment

# 获取指定地址所在的段
def get_segment(address):
    seg = ida_segment.getseg(address)
    if seg:
        return seg
    else:
        print("No segment found at address: 0x{:X}".format(address))
        return None
def has_execution_permission(segment):
    if segment.perm & ida_segment.SEGPERM_EXEC:
        return True
    else:
        return False

skip_func=['__libc_csu_init', 
             '__libc_csu_fini', 
             '_fini',
             '__do_global_dtors_aux',
             '_start',
             '_init']
flag=False
for seg in idautils.Segments():
    segname = idc.get_segm_name(seg)
    segstart = idc.get_segm_start(seg)
    segend   = idc.get_segm_end(seg)
    seg_address=get_segment(seg)
    if not flag:
        if has_execution_permission(seg_address):
            base=segstart
            flag=True
    else:
        if has_execution_permission(seg_address):
            for funcaddr in Functions(segstart,segend):
                funname = idc.get_func_name(funcaddr)
                if funname not in skip_func:
                    offset=funcaddr-base
                    print(offset)
                    #print(hex(offset))
                    # print(segname)
                    # print(funname)



# print(hex(base))
            #print(offset)

            
