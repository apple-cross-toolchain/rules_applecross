import re,sys
src=open(sys.argv[1]).read()
DL='target datalayout = "e-p:64:64:64-i1:8:8-i8:8:8-i16:16:16-i32:32:32-i64:64:64-f32:32:32-f64:64:64-v16:16:16-v24:32:32-v32:32:32-v48:64:64-v64:64:64-v96:128:128-v128:128:128-v192:256:256-v256:256:256-v512:512:512-v1024:1024:1024-n8:16:32"'
src=re.sub(r'target datalayout = ".*"',DL,src)
src=re.sub(r'target triple = ".*"','target triple = "air64_v29-apple-ios27.2.0"',src)
src=src.replace('dso_local spir_func ','')
src=src.replace('float addrspace(1)* nocapture noundef %0','float addrspace(1)* nocapture noundef "air-buffer-no-alias" %0')
# drop clang's module flags / ident, append AIR metadata
src=re.sub(r'!llvm\.module\.flags = .*\n','',src); src=re.sub(r'!llvm\.ident = .*\n','',src)
src=re.sub(r'^!0 = .*\n|^!1 = .*\n|^!2 = .*\n','',src,flags=re.M)
src=src.replace('!tbaa !3','!tbaa !103')
src=re.sub(r'^!3 = (.*)$',lambda m:'!103 = '+m.group(1).replace('!4','!104'),src,flags=re.M)
src=re.sub(r'^!4 = (.*)$',lambda m:'!104 = '+m.group(1).replace('!5','!105'),src,flags=re.M)
src=re.sub(r'^!5 = (.*)$',lambda m:'!105 = '+m.group(1).replace('!6','!106'),src,flags=re.M)
src=re.sub(r'^!6 = (.*)$',lambda m:'!106 = '+m.group(1),src,flags=re.M)
src+='''
!llvm.module.flags = !{!0, !1, !2, !3, !4, !5, !6, !7, !8}
!air.kernel = !{!9}
!air.compile_options = !{!14, !15, !16}
!air.version = !{!18}
!air.language_version = !{!19}
!0 = !{i32 2, !"SDK Version", [2 x i32] [i32 27, i32 2]}
!1 = !{i32 1, !"wchar_size", i32 4}
!2 = !{i32 7, !"frame-pointer", i32 2}
!3 = !{i32 7, !"air.max_device_buffers", i32 31}
!4 = !{i32 7, !"air.max_constant_buffers", i32 31}
!5 = !{i32 7, !"air.max_threadgroup_buffers", i32 31}
!6 = !{i32 7, !"air.max_textures", i32 128}
!7 = !{i32 7, !"air.max_read_write_textures", i32 8}
!8 = !{i32 7, !"air.max_samplers", i32 16}
!9 = !{void (float addrspace(1)*, i32)* @k, !10, !11}
!10 = !{}
!11 = !{!12, !13}
!12 = !{i32 0, !"air.buffer", !"air.location_index", i32 0, i32 1, !"air.read_write", !"air.address_space", i32 1, !"air.arg_type_size", i32 4, !"air.arg_type_align_size", i32 4, !"air.arg_type_name", !"float", !"air.arg_name", !"a"}
!13 = !{i32 1, !"air.thread_position_in_grid", !"air.arg_type_name", !"uint", !"air.arg_name", !"i"}
!14 = !{!"air.compile.denorms_disable"}
!15 = !{!"air.compile.fast_math_enable"}
!16 = !{!"air.compile.framebuffer_fetch_enable"}
!18 = !{i32 2, i32 9, i32 0}
!19 = !{!"Metal", i32 4, i32 1, i32 0}
'''
open(sys.argv[2],'w').write(src)
