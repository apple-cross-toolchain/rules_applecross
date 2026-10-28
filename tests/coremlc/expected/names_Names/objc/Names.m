//
// Names.m
//
// This file was automatically generated and should not be edited.
//

#if !__has_feature(objc_arc)
#error This file must be compiled with automatic reference counting enabled (-fobjc-arc)
#endif

#import "Names.h"

@implementation NamesInput

- (instancetype)initWithX:(MLMultiArray *)x {
    self = [super init];
    if (self) {
        _x = x;
    }
    return self;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"x"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"x"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.x];
    }
    return nil;
}

@end

@implementation NamesOutput

- (instancetype)initWithOut:(MLMultiArray *)out_ var_:(MLMultiArray *)var_ in_:(MLMultiArray *)in_ inout_:(MLMultiArray *)inout_ self_:(MLMultiArray *)self_ class_:(MLMultiArray *)class_ func_workaround:(MLMultiArray *)func_workaround let_:(MLMultiArray *)let_ import_:(MLMultiArray *)import_ return_:(MLMultiArray *)return_ if_:(MLMultiArray *)if_ for_:(MLMultiArray *)for_ while_:(MLMultiArray *)while_ struct_:(MLMultiArray *)struct_ protocol_:(MLMultiArray *)protocol_ model:(MLMultiArray *)model input:(MLMultiArray *)input output:(MLMultiArray *)output features:(MLMultiArray *)features featureNames:(MLMultiArray *)featureNames provider:(MLMultiArray *)provider id_:(MLMultiArray *)id_ int_:(MLMultiArray *)int_ float_:(MLMultiArray *)float_ double_:(MLMultiArray *)double_ char_:(MLMultiArray *)char_ void_:(MLMultiArray *)void_ bool_workaround:(MLMultiArray *)bool_workaround BOOL_:(MLMultiArray *)BOOL_ YES_:(MLMultiArray *)YES_ NO_:(MLMultiArray *)NO_ nil_:(MLMultiArray *)nil_ new:(MLMultiArray *)new default_:(MLMultiArray *)default_ switch_:(MLMultiArray *)switch_ case_:(MLMultiArray *)case_ operator_:(MLMultiArray *)operator_ typealias_:(MLMultiArray *)typealias_ extension_:(MLMultiArray *)extension_ init_:(MLMultiArray *)init_ deinit_:(MLMultiArray *)deinit_ subscript_:(MLMultiArray *)subscript_ static_:(MLMultiArray *)static_ public_:(MLMultiArray *)public_ private_:(MLMultiArray *)private_ internal_:(MLMultiArray *)internal_ true_workaround:(MLMultiArray *)true_workaround false_workaround:(MLMultiArray *)false_workaround Type_:(MLMultiArray *)Type_ Protocol_:(MLMultiArray *)Protocol_ super_:(MLMultiArray *)super_ where_:(MLMultiArray *)where_ as_:(MLMultiArray *)as_ is_:(MLMultiArray *)is_ try:(MLMultiArray *)try throw:(MLMultiArray *)throw throws:(MLMultiArray *)throws rethrows:(MLMultiArray *)rethrows async:(MLMultiArray *)async await:(MLMultiArray *)await defer:(MLMultiArray *)defer guard:(MLMultiArray *)guard repeat:(MLMultiArray *)repeat break_:(MLMultiArray *)break_ continue_:(MLMultiArray *)continue_ fallthrough_:(MLMultiArray *)fallthrough_ associatedtype:(MLMultiArray *)associatedtype enum_:(MLMultiArray *)enum_ union_:(MLMultiArray *)union_ const_:(MLMultiArray *)const_ signed_:(MLMultiArray *)signed_ unsigned_:(MLMultiArray *)unsigned_ long_:(MLMultiArray *)long_ short_:(MLMultiArray *)short_ auto_:(MLMultiArray *)auto_ register_:(MLMultiArray *)register_ extern_:(MLMultiArray *)extern_ volatile_:(MLMultiArray *)volatile_ goto_:(MLMultiArray *)goto_ sizeof_:(MLMultiArray *)sizeof_ typedef_:(MLMultiArray *)typedef_ oneway_:(MLMultiArray *)oneway_ bycopy_:(MLMultiArray *)bycopy_ byref_:(MLMultiArray *)byref_ restrict_:(MLMultiArray *)restrict_ inline_:(MLMultiArray *)inline_ template:(MLMultiArray *)template namespace:(MLMultiArray *)namespace delete:(MLMultiArray *)delete this:(MLMultiArray *)this friend:(MLMultiArray *)friend virtual:(MLMultiArray *)virtual mutable:(MLMultiArray *)mutable explicit:(MLMultiArray *)explicit export:(MLMultiArray *)export Any:(MLMultiArray *)Any Self_:(MLMultiArray *)Self_ some:(MLMultiArray *)some any_workaround:(MLMultiArray *)any_workaround do_:(MLMultiArray *)do_ else_:(MLMultiArray *)else_ open:(MLMultiArray *)open fileprivate:(MLMultiArray *)fileprivate convenience_:(MLMultiArray *)convenience_ dynamic_:(MLMultiArray *)dynamic_ final_:(MLMultiArray *)final_ lazy_:(MLMultiArray *)lazy_ optional_:(MLMultiArray *)optional_ required_:(MLMultiArray *)required_ override_:(MLMultiArray *)override_ weak_:(MLMultiArray *)weak_ unowned_:(MLMultiArray *)unowned_ mutating_:(MLMultiArray *)mutating_ nonmutating_:(MLMultiArray *)nonmutating_ indirect:(MLMultiArray *)indirect prefix_:(MLMultiArray *)prefix_ postfix_:(MLMultiArray *)postfix_ infix_:(MLMultiArray *)infix_ get_:(MLMultiArray *)get_ set_:(MLMultiArray *)set_ willSet:(MLMultiArray *)willSet didSet_:(MLMultiArray *)didSet_ NULL_:(MLMultiArray *)NULL_ Class_:(MLMultiArray *)Class_ SEL_:(MLMultiArray *)SEL_ IMP_:(MLMultiArray *)IMP_ nonatomic_:(MLMultiArray *)nonatomic_ atomic_:(MLMultiArray *)atomic_ strong:(MLMultiArray *)strong copy:(MLMultiArray *)copy assign:(MLMultiArray *)assign readonly:(MLMultiArray *)readonly readwrite:(MLMultiArray *)readwrite instancetype:(MLMultiArray *)instancetype description:(MLMultiArray *)description hash:(MLMultiArray *)hash {
    self = [super init];
    if (self) {
        _out_ = out_;
        _var_ = var_;
        _in_ = in_;
        _inout_ = inout_;
        _self_ = self_;
        _class_ = class_;
        _func_workaround = func_workaround;
        _let_ = let_;
        _import_ = import_;
        _return_ = return_;
        _if_ = if_;
        _for_ = for_;
        _while_ = while_;
        _struct_ = struct_;
        _protocol_ = protocol_;
        _model = model;
        _input = input;
        _output = output;
        _features = features;
        _featureNames = featureNames;
        _provider = provider;
        _id_ = id_;
        _int_ = int_;
        _float_ = float_;
        _double_ = double_;
        _char_ = char_;
        _void_ = void_;
        _bool_workaround = bool_workaround;
        _BOOL_ = BOOL_;
        _YES_ = YES_;
        _NO_ = NO_;
        _nil_ = nil_;
        _new = new;
        _default_ = default_;
        _switch_ = switch_;
        _case_ = case_;
        _operator_ = operator_;
        _typealias_ = typealias_;
        _extension_ = extension_;
        _init_ = init_;
        _deinit_ = deinit_;
        _subscript_ = subscript_;
        _static_ = static_;
        _public_ = public_;
        _private_ = private_;
        _internal_ = internal_;
        _true_workaround = true_workaround;
        _false_workaround = false_workaround;
        _Type_ = Type_;
        _Protocol_ = Protocol_;
        _super_ = super_;
        _where_ = where_;
        _as_ = as_;
        _is_ = is_;
        _try = try;
        _throw = throw;
        _throws = throws;
        _rethrows = rethrows;
        _async = async;
        _await = await;
        _defer = defer;
        _guard = guard;
        _repeat = repeat;
        _break_ = break_;
        _continue_ = continue_;
        _fallthrough_ = fallthrough_;
        _associatedtype = associatedtype;
        _enum_ = enum_;
        _union_ = union_;
        _const_ = const_;
        _signed_ = signed_;
        _unsigned_ = unsigned_;
        _long_ = long_;
        _short_ = short_;
        _auto_ = auto_;
        _register_ = register_;
        _extern_ = extern_;
        _volatile_ = volatile_;
        _goto_ = goto_;
        _sizeof_ = sizeof_;
        _typedef_ = typedef_;
        _oneway_ = oneway_;
        _bycopy_ = bycopy_;
        _byref_ = byref_;
        _restrict_ = restrict_;
        _inline_ = inline_;
        _template = template;
        _namespace = namespace;
        _delete = delete;
        _this = this;
        _friend = friend;
        _virtual = virtual;
        _mutable = mutable;
        _explicit = explicit;
        _export = export;
        _Any = Any;
        _Self_ = Self_;
        _some = some;
        _any_workaround = any_workaround;
        _do_ = do_;
        _else_ = else_;
        _open = open;
        _fileprivate = fileprivate;
        _convenience_ = convenience_;
        _dynamic_ = dynamic_;
        _final_ = final_;
        _lazy_ = lazy_;
        _optional_ = optional_;
        _required_ = required_;
        _override_ = override_;
        _weak_ = weak_;
        _unowned_ = unowned_;
        _mutating_ = mutating_;
        _nonmutating_ = nonmutating_;
        _indirect = indirect;
        _prefix_ = prefix_;
        _postfix_ = postfix_;
        _infix_ = infix_;
        _get_ = get_;
        _set_ = set_;
        _willSet = willSet;
        _didSet_ = didSet_;
        _NULL_ = NULL_;
        _Class_ = Class_;
        _SEL_ = SEL_;
        _IMP_ = IMP_;
        _nonatomic_ = nonatomic_;
        _atomic_ = atomic_;
        _strong = strong;
        _copy = copy;
        _assign = assign;
        _readonly = readonly;
        _readwrite = readwrite;
        _instancetype = instancetype;
        _description = description;
        _hash = hash;
    }
    return self;
}

- (NSSet<NSString *> *)featureNames {
    return [NSSet setWithArray:@[@"out", @"var", @"in", @"inout", @"self", @"class", @"func_workaround", @"let", @"import", @"return", @"if", @"for", @"while", @"struct", @"protocol", @"model", @"input", @"output", @"features", @"featureNames", @"provider", @"id", @"int", @"float", @"double", @"char", @"void", @"bool_workaround", @"BOOL", @"YES", @"NO", @"nil", @"new", @"default", @"switch", @"case", @"operator", @"typealias", @"extension", @"init", @"deinit", @"subscript", @"static", @"public", @"private", @"internal", @"true_workaround", @"false_workaround", @"Type", @"Protocol", @"super", @"where", @"as", @"is", @"try", @"throw", @"throws", @"rethrows", @"async", @"await", @"defer", @"guard", @"repeat", @"break", @"continue", @"fallthrough", @"associatedtype", @"enum", @"union", @"const", @"signed", @"unsigned", @"long", @"short", @"auto", @"register", @"extern", @"volatile", @"goto", @"sizeof", @"typedef", @"oneway", @"bycopy", @"byref", @"restrict", @"inline", @"template", @"namespace", @"delete", @"this", @"friend", @"virtual", @"mutable", @"explicit", @"export", @"Any", @"Self", @"some", @"any_workaround", @"do", @"else", @"open", @"fileprivate", @"convenience", @"dynamic", @"final", @"lazy", @"optional", @"required", @"override", @"weak", @"unowned", @"mutating", @"nonmutating", @"indirect", @"prefix", @"postfix", @"infix", @"get", @"set", @"willSet", @"didSet", @"NULL", @"Class", @"SEL", @"IMP", @"nonatomic", @"atomic", @"strong", @"copy", @"assign", @"readonly", @"readwrite", @"instancetype", @"description", @"hash"]];
}

- (nullable MLFeatureValue *)featureValueForName:(NSString *)featureName {
    if ([featureName isEqualToString:@"out"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.out_];
    }
    if ([featureName isEqualToString:@"var"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.var_];
    }
    if ([featureName isEqualToString:@"in"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.in_];
    }
    if ([featureName isEqualToString:@"inout"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.inout_];
    }
    if ([featureName isEqualToString:@"self"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.self_];
    }
    if ([featureName isEqualToString:@"class"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.class_];
    }
    if ([featureName isEqualToString:@"func_workaround"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.func_workaround];
    }
    if ([featureName isEqualToString:@"let"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.let_];
    }
    if ([featureName isEqualToString:@"import"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.import_];
    }
    if ([featureName isEqualToString:@"return"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.return_];
    }
    if ([featureName isEqualToString:@"if"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.if_];
    }
    if ([featureName isEqualToString:@"for"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.for_];
    }
    if ([featureName isEqualToString:@"while"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.while_];
    }
    if ([featureName isEqualToString:@"struct"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.struct_];
    }
    if ([featureName isEqualToString:@"protocol"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.protocol_];
    }
    if ([featureName isEqualToString:@"model"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.model];
    }
    if ([featureName isEqualToString:@"input"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.input];
    }
    if ([featureName isEqualToString:@"output"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.output];
    }
    if ([featureName isEqualToString:@"features"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.features];
    }
    if ([featureName isEqualToString:@"featureNames"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.featureNames];
    }
    if ([featureName isEqualToString:@"provider"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.provider];
    }
    if ([featureName isEqualToString:@"id"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.id_];
    }
    if ([featureName isEqualToString:@"int"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.int_];
    }
    if ([featureName isEqualToString:@"float"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.float_];
    }
    if ([featureName isEqualToString:@"double"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.double_];
    }
    if ([featureName isEqualToString:@"char"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.char_];
    }
    if ([featureName isEqualToString:@"void"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.void_];
    }
    if ([featureName isEqualToString:@"bool_workaround"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.bool_workaround];
    }
    if ([featureName isEqualToString:@"BOOL"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.BOOL_];
    }
    if ([featureName isEqualToString:@"YES"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.YES_];
    }
    if ([featureName isEqualToString:@"NO"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.NO_];
    }
    if ([featureName isEqualToString:@"nil"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.nil_];
    }
    if ([featureName isEqualToString:@"new"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.new];
    }
    if ([featureName isEqualToString:@"default"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.default_];
    }
    if ([featureName isEqualToString:@"switch"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.switch_];
    }
    if ([featureName isEqualToString:@"case"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.case_];
    }
    if ([featureName isEqualToString:@"operator"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.operator_];
    }
    if ([featureName isEqualToString:@"typealias"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.typealias_];
    }
    if ([featureName isEqualToString:@"extension"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.extension_];
    }
    if ([featureName isEqualToString:@"init"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.init_];
    }
    if ([featureName isEqualToString:@"deinit"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.deinit_];
    }
    if ([featureName isEqualToString:@"subscript"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.subscript_];
    }
    if ([featureName isEqualToString:@"static"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.static_];
    }
    if ([featureName isEqualToString:@"public"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.public_];
    }
    if ([featureName isEqualToString:@"private"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.private_];
    }
    if ([featureName isEqualToString:@"internal"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.internal_];
    }
    if ([featureName isEqualToString:@"true_workaround"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.true_workaround];
    }
    if ([featureName isEqualToString:@"false_workaround"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.false_workaround];
    }
    if ([featureName isEqualToString:@"Type"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.Type_];
    }
    if ([featureName isEqualToString:@"Protocol"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.Protocol_];
    }
    if ([featureName isEqualToString:@"super"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.super_];
    }
    if ([featureName isEqualToString:@"where"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.where_];
    }
    if ([featureName isEqualToString:@"as"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.as_];
    }
    if ([featureName isEqualToString:@"is"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.is_];
    }
    if ([featureName isEqualToString:@"try"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.try];
    }
    if ([featureName isEqualToString:@"throw"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.throw];
    }
    if ([featureName isEqualToString:@"throws"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.throws];
    }
    if ([featureName isEqualToString:@"rethrows"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.rethrows];
    }
    if ([featureName isEqualToString:@"async"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.async];
    }
    if ([featureName isEqualToString:@"await"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.await];
    }
    if ([featureName isEqualToString:@"defer"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.defer];
    }
    if ([featureName isEqualToString:@"guard"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.guard];
    }
    if ([featureName isEqualToString:@"repeat"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.repeat];
    }
    if ([featureName isEqualToString:@"break"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.break_];
    }
    if ([featureName isEqualToString:@"continue"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.continue_];
    }
    if ([featureName isEqualToString:@"fallthrough"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.fallthrough_];
    }
    if ([featureName isEqualToString:@"associatedtype"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.associatedtype];
    }
    if ([featureName isEqualToString:@"enum"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.enum_];
    }
    if ([featureName isEqualToString:@"union"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.union_];
    }
    if ([featureName isEqualToString:@"const"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.const_];
    }
    if ([featureName isEqualToString:@"signed"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.signed_];
    }
    if ([featureName isEqualToString:@"unsigned"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.unsigned_];
    }
    if ([featureName isEqualToString:@"long"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.long_];
    }
    if ([featureName isEqualToString:@"short"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.short_];
    }
    if ([featureName isEqualToString:@"auto"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.auto_];
    }
    if ([featureName isEqualToString:@"register"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.register_];
    }
    if ([featureName isEqualToString:@"extern"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.extern_];
    }
    if ([featureName isEqualToString:@"volatile"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.volatile_];
    }
    if ([featureName isEqualToString:@"goto"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.goto_];
    }
    if ([featureName isEqualToString:@"sizeof"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.sizeof_];
    }
    if ([featureName isEqualToString:@"typedef"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.typedef_];
    }
    if ([featureName isEqualToString:@"oneway"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.oneway_];
    }
    if ([featureName isEqualToString:@"bycopy"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.bycopy_];
    }
    if ([featureName isEqualToString:@"byref"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.byref_];
    }
    if ([featureName isEqualToString:@"restrict"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.restrict_];
    }
    if ([featureName isEqualToString:@"inline"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.inline_];
    }
    if ([featureName isEqualToString:@"template"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.template];
    }
    if ([featureName isEqualToString:@"namespace"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.namespace];
    }
    if ([featureName isEqualToString:@"delete"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.delete];
    }
    if ([featureName isEqualToString:@"this"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.this];
    }
    if ([featureName isEqualToString:@"friend"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.friend];
    }
    if ([featureName isEqualToString:@"virtual"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.virtual];
    }
    if ([featureName isEqualToString:@"mutable"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.mutable];
    }
    if ([featureName isEqualToString:@"explicit"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.explicit];
    }
    if ([featureName isEqualToString:@"export"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.export];
    }
    if ([featureName isEqualToString:@"Any"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.Any];
    }
    if ([featureName isEqualToString:@"Self"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.Self_];
    }
    if ([featureName isEqualToString:@"some"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.some];
    }
    if ([featureName isEqualToString:@"any_workaround"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.any_workaround];
    }
    if ([featureName isEqualToString:@"do"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.do_];
    }
    if ([featureName isEqualToString:@"else"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.else_];
    }
    if ([featureName isEqualToString:@"open"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.open];
    }
    if ([featureName isEqualToString:@"fileprivate"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.fileprivate];
    }
    if ([featureName isEqualToString:@"convenience"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.convenience_];
    }
    if ([featureName isEqualToString:@"dynamic"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.dynamic_];
    }
    if ([featureName isEqualToString:@"final"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.final_];
    }
    if ([featureName isEqualToString:@"lazy"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.lazy_];
    }
    if ([featureName isEqualToString:@"optional"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.optional_];
    }
    if ([featureName isEqualToString:@"required"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.required_];
    }
    if ([featureName isEqualToString:@"override"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.override_];
    }
    if ([featureName isEqualToString:@"weak"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.weak_];
    }
    if ([featureName isEqualToString:@"unowned"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.unowned_];
    }
    if ([featureName isEqualToString:@"mutating"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.mutating_];
    }
    if ([featureName isEqualToString:@"nonmutating"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.nonmutating_];
    }
    if ([featureName isEqualToString:@"indirect"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.indirect];
    }
    if ([featureName isEqualToString:@"prefix"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.prefix_];
    }
    if ([featureName isEqualToString:@"postfix"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.postfix_];
    }
    if ([featureName isEqualToString:@"infix"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.infix_];
    }
    if ([featureName isEqualToString:@"get"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.get_];
    }
    if ([featureName isEqualToString:@"set"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.set_];
    }
    if ([featureName isEqualToString:@"willSet"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.willSet];
    }
    if ([featureName isEqualToString:@"didSet"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.didSet_];
    }
    if ([featureName isEqualToString:@"NULL"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.NULL_];
    }
    if ([featureName isEqualToString:@"Class"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.Class_];
    }
    if ([featureName isEqualToString:@"SEL"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.SEL_];
    }
    if ([featureName isEqualToString:@"IMP"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.IMP_];
    }
    if ([featureName isEqualToString:@"nonatomic"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.nonatomic_];
    }
    if ([featureName isEqualToString:@"atomic"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.atomic_];
    }
    if ([featureName isEqualToString:@"strong"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.strong];
    }
    if ([featureName isEqualToString:@"copy"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.copy];
    }
    if ([featureName isEqualToString:@"assign"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.assign];
    }
    if ([featureName isEqualToString:@"readonly"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.readonly];
    }
    if ([featureName isEqualToString:@"readwrite"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.readwrite];
    }
    if ([featureName isEqualToString:@"instancetype"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.instancetype];
    }
    if ([featureName isEqualToString:@"description"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.description];
    }
    if ([featureName isEqualToString:@"hash"]) {
        return [MLFeatureValue featureValueWithMultiArray:self.hash];
    }
    return nil;
}

@end

@implementation Names


/**
    URL of the underlying .mlmodelc directory.
*/
+ (nullable NSURL *)URLOfModelInThisBundle {
    NSString *assetPath = [[NSBundle bundleForClass:[self class]] pathForResource:@"Names" ofType:@"mlmodelc"];
    if (nil == assetPath) { os_log_error(OS_LOG_DEFAULT, "Could not load Names.mlmodelc in the bundle resource"); return nil; }
    return [NSURL fileURLWithPath:assetPath];
}


/**
    Initialize Names instance from an existing MLModel object.

    Usually the application does not use this initializer unless it makes a subclass of Names.
    Such application may want to use `-[MLModel initWithContentsOfURL:configuration:error:]` and `+URLOfModelInThisBundle` to create a MLModel object to pass-in.
*/
- (instancetype)initWithMLModel:(MLModel *)model {
    if (model == nil) {
        return nil;
    }
    self = [super init];
    if (self != nil) {
        _model = model;
    }
    return self;
}


/**
    Initialize Names instance with the model in this bundle.
*/
- (nullable instancetype)init {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle error:nil];
}


/**
    Initialize Names instance with the model in this bundle.

    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithConfiguration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self initWithContentsOfURL:(NSURL * _Nonnull)self.class.URLOfModelInThisBundle configuration:configuration error:error];
}


/**
    Initialize Names instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for Names.
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Initialize Names instance from the model URL.

    @param modelURL URL to the .mlmodelc directory for Names.
    @param configuration The model configuration object
    @param error If an error occurs, upon return contains an NSError object that describes the problem. If you are not interested in possible errors, pass in NULL.
*/
- (nullable instancetype)initWithContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    MLModel *model = [MLModel modelWithContentsOfURL:modelURL configuration:configuration error:error];
    if (model == nil) { return nil; }
    return [self initWithMLModel:model];
}


/**
    Construct Names instance asynchronously with configuration.
    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid Names instance or NSError object.
*/
+ (void)loadWithConfiguration:(MLModelConfiguration *)configuration completionHandler:(void (^)(Names * _Nullable model, NSError * _Nullable error))handler {
    [self loadContentsOfURL:(NSURL * _Nonnull)[self URLOfModelInThisBundle]
              configuration:configuration
          completionHandler:handler];
}


/**
    Construct Names instance asynchronously with URL of .mlmodelc directory and optional configuration.

    Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

    @param modelURL The model URL.
    @param configuration The model configuration
    @param handler When the model load completes successfully or unsuccessfully, the completion handler is invoked with a valid Names instance or NSError object.
*/
+ (void)loadContentsOfURL:(NSURL *)modelURL configuration:(MLModelConfiguration *)configuration completionHandler:(void (^)(Names * _Nullable model, NSError * _Nullable error))handler {
    [MLModel loadContentsOfURL:modelURL
                 configuration:configuration
             completionHandler:^(MLModel *model, NSError *error) {
        if (model != nil) {
            Names *typedModel = [[Names alloc] initWithMLModel:model];
            handler(typedModel, nil);
        } else {
            handler(nil, error);
        }
    }];
}

- (nullable NamesOutput *)predictionFromFeatures:(NamesInput *)input error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    return [self predictionFromFeatures:input options:[[MLPredictionOptions alloc] init] error:error];
}

- (nullable NamesOutput *)predictionFromFeatures:(NamesInput *)input options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLFeatureProvider> outFeatures = [self.model predictionFromFeatures:input options:options error:error];
    if (!outFeatures) { return nil; }
    return [[NamesOutput alloc] initWithOut:(MLMultiArray *)[outFeatures featureValueForName:@"out"].multiArrayValue var_:(MLMultiArray *)[outFeatures featureValueForName:@"var"].multiArrayValue in_:(MLMultiArray *)[outFeatures featureValueForName:@"in"].multiArrayValue inout_:(MLMultiArray *)[outFeatures featureValueForName:@"inout"].multiArrayValue self_:(MLMultiArray *)[outFeatures featureValueForName:@"self"].multiArrayValue class_:(MLMultiArray *)[outFeatures featureValueForName:@"class"].multiArrayValue func_workaround:(MLMultiArray *)[outFeatures featureValueForName:@"func_workaround"].multiArrayValue let_:(MLMultiArray *)[outFeatures featureValueForName:@"let"].multiArrayValue import_:(MLMultiArray *)[outFeatures featureValueForName:@"import"].multiArrayValue return_:(MLMultiArray *)[outFeatures featureValueForName:@"return"].multiArrayValue if_:(MLMultiArray *)[outFeatures featureValueForName:@"if"].multiArrayValue for_:(MLMultiArray *)[outFeatures featureValueForName:@"for"].multiArrayValue while_:(MLMultiArray *)[outFeatures featureValueForName:@"while"].multiArrayValue struct_:(MLMultiArray *)[outFeatures featureValueForName:@"struct"].multiArrayValue protocol_:(MLMultiArray *)[outFeatures featureValueForName:@"protocol"].multiArrayValue model:(MLMultiArray *)[outFeatures featureValueForName:@"model"].multiArrayValue input:(MLMultiArray *)[outFeatures featureValueForName:@"input"].multiArrayValue output:(MLMultiArray *)[outFeatures featureValueForName:@"output"].multiArrayValue features:(MLMultiArray *)[outFeatures featureValueForName:@"features"].multiArrayValue featureNames:(MLMultiArray *)[outFeatures featureValueForName:@"featureNames"].multiArrayValue provider:(MLMultiArray *)[outFeatures featureValueForName:@"provider"].multiArrayValue id_:(MLMultiArray *)[outFeatures featureValueForName:@"id"].multiArrayValue int_:(MLMultiArray *)[outFeatures featureValueForName:@"int"].multiArrayValue float_:(MLMultiArray *)[outFeatures featureValueForName:@"float"].multiArrayValue double_:(MLMultiArray *)[outFeatures featureValueForName:@"double"].multiArrayValue char_:(MLMultiArray *)[outFeatures featureValueForName:@"char"].multiArrayValue void_:(MLMultiArray *)[outFeatures featureValueForName:@"void"].multiArrayValue bool_workaround:(MLMultiArray *)[outFeatures featureValueForName:@"bool_workaround"].multiArrayValue BOOL_:(MLMultiArray *)[outFeatures featureValueForName:@"BOOL"].multiArrayValue YES_:(MLMultiArray *)[outFeatures featureValueForName:@"YES"].multiArrayValue NO_:(MLMultiArray *)[outFeatures featureValueForName:@"NO"].multiArrayValue nil_:(MLMultiArray *)[outFeatures featureValueForName:@"nil"].multiArrayValue new:(MLMultiArray *)[outFeatures featureValueForName:@"new"].multiArrayValue default_:(MLMultiArray *)[outFeatures featureValueForName:@"default"].multiArrayValue switch_:(MLMultiArray *)[outFeatures featureValueForName:@"switch"].multiArrayValue case_:(MLMultiArray *)[outFeatures featureValueForName:@"case"].multiArrayValue operator_:(MLMultiArray *)[outFeatures featureValueForName:@"operator"].multiArrayValue typealias_:(MLMultiArray *)[outFeatures featureValueForName:@"typealias"].multiArrayValue extension_:(MLMultiArray *)[outFeatures featureValueForName:@"extension"].multiArrayValue init_:(MLMultiArray *)[outFeatures featureValueForName:@"init"].multiArrayValue deinit_:(MLMultiArray *)[outFeatures featureValueForName:@"deinit"].multiArrayValue subscript_:(MLMultiArray *)[outFeatures featureValueForName:@"subscript"].multiArrayValue static_:(MLMultiArray *)[outFeatures featureValueForName:@"static"].multiArrayValue public_:(MLMultiArray *)[outFeatures featureValueForName:@"public"].multiArrayValue private_:(MLMultiArray *)[outFeatures featureValueForName:@"private"].multiArrayValue internal_:(MLMultiArray *)[outFeatures featureValueForName:@"internal"].multiArrayValue true_workaround:(MLMultiArray *)[outFeatures featureValueForName:@"true_workaround"].multiArrayValue false_workaround:(MLMultiArray *)[outFeatures featureValueForName:@"false_workaround"].multiArrayValue Type_:(MLMultiArray *)[outFeatures featureValueForName:@"Type"].multiArrayValue Protocol_:(MLMultiArray *)[outFeatures featureValueForName:@"Protocol"].multiArrayValue super_:(MLMultiArray *)[outFeatures featureValueForName:@"super"].multiArrayValue where_:(MLMultiArray *)[outFeatures featureValueForName:@"where"].multiArrayValue as_:(MLMultiArray *)[outFeatures featureValueForName:@"as"].multiArrayValue is_:(MLMultiArray *)[outFeatures featureValueForName:@"is"].multiArrayValue try:(MLMultiArray *)[outFeatures featureValueForName:@"try"].multiArrayValue throw:(MLMultiArray *)[outFeatures featureValueForName:@"throw"].multiArrayValue throws:(MLMultiArray *)[outFeatures featureValueForName:@"throws"].multiArrayValue rethrows:(MLMultiArray *)[outFeatures featureValueForName:@"rethrows"].multiArrayValue async:(MLMultiArray *)[outFeatures featureValueForName:@"async"].multiArrayValue await:(MLMultiArray *)[outFeatures featureValueForName:@"await"].multiArrayValue defer:(MLMultiArray *)[outFeatures featureValueForName:@"defer"].multiArrayValue guard:(MLMultiArray *)[outFeatures featureValueForName:@"guard"].multiArrayValue repeat:(MLMultiArray *)[outFeatures featureValueForName:@"repeat"].multiArrayValue break_:(MLMultiArray *)[outFeatures featureValueForName:@"break"].multiArrayValue continue_:(MLMultiArray *)[outFeatures featureValueForName:@"continue"].multiArrayValue fallthrough_:(MLMultiArray *)[outFeatures featureValueForName:@"fallthrough"].multiArrayValue associatedtype:(MLMultiArray *)[outFeatures featureValueForName:@"associatedtype"].multiArrayValue enum_:(MLMultiArray *)[outFeatures featureValueForName:@"enum"].multiArrayValue union_:(MLMultiArray *)[outFeatures featureValueForName:@"union"].multiArrayValue const_:(MLMultiArray *)[outFeatures featureValueForName:@"const"].multiArrayValue signed_:(MLMultiArray *)[outFeatures featureValueForName:@"signed"].multiArrayValue unsigned_:(MLMultiArray *)[outFeatures featureValueForName:@"unsigned"].multiArrayValue long_:(MLMultiArray *)[outFeatures featureValueForName:@"long"].multiArrayValue short_:(MLMultiArray *)[outFeatures featureValueForName:@"short"].multiArrayValue auto_:(MLMultiArray *)[outFeatures featureValueForName:@"auto"].multiArrayValue register_:(MLMultiArray *)[outFeatures featureValueForName:@"register"].multiArrayValue extern_:(MLMultiArray *)[outFeatures featureValueForName:@"extern"].multiArrayValue volatile_:(MLMultiArray *)[outFeatures featureValueForName:@"volatile"].multiArrayValue goto_:(MLMultiArray *)[outFeatures featureValueForName:@"goto"].multiArrayValue sizeof_:(MLMultiArray *)[outFeatures featureValueForName:@"sizeof"].multiArrayValue typedef_:(MLMultiArray *)[outFeatures featureValueForName:@"typedef"].multiArrayValue oneway_:(MLMultiArray *)[outFeatures featureValueForName:@"oneway"].multiArrayValue bycopy_:(MLMultiArray *)[outFeatures featureValueForName:@"bycopy"].multiArrayValue byref_:(MLMultiArray *)[outFeatures featureValueForName:@"byref"].multiArrayValue restrict_:(MLMultiArray *)[outFeatures featureValueForName:@"restrict"].multiArrayValue inline_:(MLMultiArray *)[outFeatures featureValueForName:@"inline"].multiArrayValue template:(MLMultiArray *)[outFeatures featureValueForName:@"template"].multiArrayValue namespace:(MLMultiArray *)[outFeatures featureValueForName:@"namespace"].multiArrayValue delete:(MLMultiArray *)[outFeatures featureValueForName:@"delete"].multiArrayValue this:(MLMultiArray *)[outFeatures featureValueForName:@"this"].multiArrayValue friend:(MLMultiArray *)[outFeatures featureValueForName:@"friend"].multiArrayValue virtual:(MLMultiArray *)[outFeatures featureValueForName:@"virtual"].multiArrayValue mutable:(MLMultiArray *)[outFeatures featureValueForName:@"mutable"].multiArrayValue explicit:(MLMultiArray *)[outFeatures featureValueForName:@"explicit"].multiArrayValue export:(MLMultiArray *)[outFeatures featureValueForName:@"export"].multiArrayValue Any:(MLMultiArray *)[outFeatures featureValueForName:@"Any"].multiArrayValue Self_:(MLMultiArray *)[outFeatures featureValueForName:@"Self"].multiArrayValue some:(MLMultiArray *)[outFeatures featureValueForName:@"some"].multiArrayValue any_workaround:(MLMultiArray *)[outFeatures featureValueForName:@"any_workaround"].multiArrayValue do_:(MLMultiArray *)[outFeatures featureValueForName:@"do"].multiArrayValue else_:(MLMultiArray *)[outFeatures featureValueForName:@"else"].multiArrayValue open:(MLMultiArray *)[outFeatures featureValueForName:@"open"].multiArrayValue fileprivate:(MLMultiArray *)[outFeatures featureValueForName:@"fileprivate"].multiArrayValue convenience_:(MLMultiArray *)[outFeatures featureValueForName:@"convenience"].multiArrayValue dynamic_:(MLMultiArray *)[outFeatures featureValueForName:@"dynamic"].multiArrayValue final_:(MLMultiArray *)[outFeatures featureValueForName:@"final"].multiArrayValue lazy_:(MLMultiArray *)[outFeatures featureValueForName:@"lazy"].multiArrayValue optional_:(MLMultiArray *)[outFeatures featureValueForName:@"optional"].multiArrayValue required_:(MLMultiArray *)[outFeatures featureValueForName:@"required"].multiArrayValue override_:(MLMultiArray *)[outFeatures featureValueForName:@"override"].multiArrayValue weak_:(MLMultiArray *)[outFeatures featureValueForName:@"weak"].multiArrayValue unowned_:(MLMultiArray *)[outFeatures featureValueForName:@"unowned"].multiArrayValue mutating_:(MLMultiArray *)[outFeatures featureValueForName:@"mutating"].multiArrayValue nonmutating_:(MLMultiArray *)[outFeatures featureValueForName:@"nonmutating"].multiArrayValue indirect:(MLMultiArray *)[outFeatures featureValueForName:@"indirect"].multiArrayValue prefix_:(MLMultiArray *)[outFeatures featureValueForName:@"prefix"].multiArrayValue postfix_:(MLMultiArray *)[outFeatures featureValueForName:@"postfix"].multiArrayValue infix_:(MLMultiArray *)[outFeatures featureValueForName:@"infix"].multiArrayValue get_:(MLMultiArray *)[outFeatures featureValueForName:@"get"].multiArrayValue set_:(MLMultiArray *)[outFeatures featureValueForName:@"set"].multiArrayValue willSet:(MLMultiArray *)[outFeatures featureValueForName:@"willSet"].multiArrayValue didSet_:(MLMultiArray *)[outFeatures featureValueForName:@"didSet"].multiArrayValue NULL_:(MLMultiArray *)[outFeatures featureValueForName:@"NULL"].multiArrayValue Class_:(MLMultiArray *)[outFeatures featureValueForName:@"Class"].multiArrayValue SEL_:(MLMultiArray *)[outFeatures featureValueForName:@"SEL"].multiArrayValue IMP_:(MLMultiArray *)[outFeatures featureValueForName:@"IMP"].multiArrayValue nonatomic_:(MLMultiArray *)[outFeatures featureValueForName:@"nonatomic"].multiArrayValue atomic_:(MLMultiArray *)[outFeatures featureValueForName:@"atomic"].multiArrayValue strong:(MLMultiArray *)[outFeatures featureValueForName:@"strong"].multiArrayValue copy:(MLMultiArray *)[outFeatures featureValueForName:@"copy"].multiArrayValue assign:(MLMultiArray *)[outFeatures featureValueForName:@"assign"].multiArrayValue readonly:(MLMultiArray *)[outFeatures featureValueForName:@"readonly"].multiArrayValue readwrite:(MLMultiArray *)[outFeatures featureValueForName:@"readwrite"].multiArrayValue instancetype:(MLMultiArray *)[outFeatures featureValueForName:@"instancetype"].multiArrayValue description:(MLMultiArray *)[outFeatures featureValueForName:@"description"].multiArrayValue hash:(MLMultiArray *)[outFeatures featureValueForName:@"hash"].multiArrayValue];
}

- (void)predictionFromFeatures:(NamesInput *)input completionHandler:(void (^)(NamesOutput * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            NamesOutput *output = [[NamesOutput alloc] initWithOut:(MLMultiArray *)[prediction featureValueForName:@"out"].multiArrayValue var_:(MLMultiArray *)[prediction featureValueForName:@"var"].multiArrayValue in_:(MLMultiArray *)[prediction featureValueForName:@"in"].multiArrayValue inout_:(MLMultiArray *)[prediction featureValueForName:@"inout"].multiArrayValue self_:(MLMultiArray *)[prediction featureValueForName:@"self"].multiArrayValue class_:(MLMultiArray *)[prediction featureValueForName:@"class"].multiArrayValue func_workaround:(MLMultiArray *)[prediction featureValueForName:@"func_workaround"].multiArrayValue let_:(MLMultiArray *)[prediction featureValueForName:@"let"].multiArrayValue import_:(MLMultiArray *)[prediction featureValueForName:@"import"].multiArrayValue return_:(MLMultiArray *)[prediction featureValueForName:@"return"].multiArrayValue if_:(MLMultiArray *)[prediction featureValueForName:@"if"].multiArrayValue for_:(MLMultiArray *)[prediction featureValueForName:@"for"].multiArrayValue while_:(MLMultiArray *)[prediction featureValueForName:@"while"].multiArrayValue struct_:(MLMultiArray *)[prediction featureValueForName:@"struct"].multiArrayValue protocol_:(MLMultiArray *)[prediction featureValueForName:@"protocol"].multiArrayValue model:(MLMultiArray *)[prediction featureValueForName:@"model"].multiArrayValue input:(MLMultiArray *)[prediction featureValueForName:@"input"].multiArrayValue output:(MLMultiArray *)[prediction featureValueForName:@"output"].multiArrayValue features:(MLMultiArray *)[prediction featureValueForName:@"features"].multiArrayValue featureNames:(MLMultiArray *)[prediction featureValueForName:@"featureNames"].multiArrayValue provider:(MLMultiArray *)[prediction featureValueForName:@"provider"].multiArrayValue id_:(MLMultiArray *)[prediction featureValueForName:@"id"].multiArrayValue int_:(MLMultiArray *)[prediction featureValueForName:@"int"].multiArrayValue float_:(MLMultiArray *)[prediction featureValueForName:@"float"].multiArrayValue double_:(MLMultiArray *)[prediction featureValueForName:@"double"].multiArrayValue char_:(MLMultiArray *)[prediction featureValueForName:@"char"].multiArrayValue void_:(MLMultiArray *)[prediction featureValueForName:@"void"].multiArrayValue bool_workaround:(MLMultiArray *)[prediction featureValueForName:@"bool_workaround"].multiArrayValue BOOL_:(MLMultiArray *)[prediction featureValueForName:@"BOOL"].multiArrayValue YES_:(MLMultiArray *)[prediction featureValueForName:@"YES"].multiArrayValue NO_:(MLMultiArray *)[prediction featureValueForName:@"NO"].multiArrayValue nil_:(MLMultiArray *)[prediction featureValueForName:@"nil"].multiArrayValue new:(MLMultiArray *)[prediction featureValueForName:@"new"].multiArrayValue default_:(MLMultiArray *)[prediction featureValueForName:@"default"].multiArrayValue switch_:(MLMultiArray *)[prediction featureValueForName:@"switch"].multiArrayValue case_:(MLMultiArray *)[prediction featureValueForName:@"case"].multiArrayValue operator_:(MLMultiArray *)[prediction featureValueForName:@"operator"].multiArrayValue typealias_:(MLMultiArray *)[prediction featureValueForName:@"typealias"].multiArrayValue extension_:(MLMultiArray *)[prediction featureValueForName:@"extension"].multiArrayValue init_:(MLMultiArray *)[prediction featureValueForName:@"init"].multiArrayValue deinit_:(MLMultiArray *)[prediction featureValueForName:@"deinit"].multiArrayValue subscript_:(MLMultiArray *)[prediction featureValueForName:@"subscript"].multiArrayValue static_:(MLMultiArray *)[prediction featureValueForName:@"static"].multiArrayValue public_:(MLMultiArray *)[prediction featureValueForName:@"public"].multiArrayValue private_:(MLMultiArray *)[prediction featureValueForName:@"private"].multiArrayValue internal_:(MLMultiArray *)[prediction featureValueForName:@"internal"].multiArrayValue true_workaround:(MLMultiArray *)[prediction featureValueForName:@"true_workaround"].multiArrayValue false_workaround:(MLMultiArray *)[prediction featureValueForName:@"false_workaround"].multiArrayValue Type_:(MLMultiArray *)[prediction featureValueForName:@"Type"].multiArrayValue Protocol_:(MLMultiArray *)[prediction featureValueForName:@"Protocol"].multiArrayValue super_:(MLMultiArray *)[prediction featureValueForName:@"super"].multiArrayValue where_:(MLMultiArray *)[prediction featureValueForName:@"where"].multiArrayValue as_:(MLMultiArray *)[prediction featureValueForName:@"as"].multiArrayValue is_:(MLMultiArray *)[prediction featureValueForName:@"is"].multiArrayValue try:(MLMultiArray *)[prediction featureValueForName:@"try"].multiArrayValue throw:(MLMultiArray *)[prediction featureValueForName:@"throw"].multiArrayValue throws:(MLMultiArray *)[prediction featureValueForName:@"throws"].multiArrayValue rethrows:(MLMultiArray *)[prediction featureValueForName:@"rethrows"].multiArrayValue async:(MLMultiArray *)[prediction featureValueForName:@"async"].multiArrayValue await:(MLMultiArray *)[prediction featureValueForName:@"await"].multiArrayValue defer:(MLMultiArray *)[prediction featureValueForName:@"defer"].multiArrayValue guard:(MLMultiArray *)[prediction featureValueForName:@"guard"].multiArrayValue repeat:(MLMultiArray *)[prediction featureValueForName:@"repeat"].multiArrayValue break_:(MLMultiArray *)[prediction featureValueForName:@"break"].multiArrayValue continue_:(MLMultiArray *)[prediction featureValueForName:@"continue"].multiArrayValue fallthrough_:(MLMultiArray *)[prediction featureValueForName:@"fallthrough"].multiArrayValue associatedtype:(MLMultiArray *)[prediction featureValueForName:@"associatedtype"].multiArrayValue enum_:(MLMultiArray *)[prediction featureValueForName:@"enum"].multiArrayValue union_:(MLMultiArray *)[prediction featureValueForName:@"union"].multiArrayValue const_:(MLMultiArray *)[prediction featureValueForName:@"const"].multiArrayValue signed_:(MLMultiArray *)[prediction featureValueForName:@"signed"].multiArrayValue unsigned_:(MLMultiArray *)[prediction featureValueForName:@"unsigned"].multiArrayValue long_:(MLMultiArray *)[prediction featureValueForName:@"long"].multiArrayValue short_:(MLMultiArray *)[prediction featureValueForName:@"short"].multiArrayValue auto_:(MLMultiArray *)[prediction featureValueForName:@"auto"].multiArrayValue register_:(MLMultiArray *)[prediction featureValueForName:@"register"].multiArrayValue extern_:(MLMultiArray *)[prediction featureValueForName:@"extern"].multiArrayValue volatile_:(MLMultiArray *)[prediction featureValueForName:@"volatile"].multiArrayValue goto_:(MLMultiArray *)[prediction featureValueForName:@"goto"].multiArrayValue sizeof_:(MLMultiArray *)[prediction featureValueForName:@"sizeof"].multiArrayValue typedef_:(MLMultiArray *)[prediction featureValueForName:@"typedef"].multiArrayValue oneway_:(MLMultiArray *)[prediction featureValueForName:@"oneway"].multiArrayValue bycopy_:(MLMultiArray *)[prediction featureValueForName:@"bycopy"].multiArrayValue byref_:(MLMultiArray *)[prediction featureValueForName:@"byref"].multiArrayValue restrict_:(MLMultiArray *)[prediction featureValueForName:@"restrict"].multiArrayValue inline_:(MLMultiArray *)[prediction featureValueForName:@"inline"].multiArrayValue template:(MLMultiArray *)[prediction featureValueForName:@"template"].multiArrayValue namespace:(MLMultiArray *)[prediction featureValueForName:@"namespace"].multiArrayValue delete:(MLMultiArray *)[prediction featureValueForName:@"delete"].multiArrayValue this:(MLMultiArray *)[prediction featureValueForName:@"this"].multiArrayValue friend:(MLMultiArray *)[prediction featureValueForName:@"friend"].multiArrayValue virtual:(MLMultiArray *)[prediction featureValueForName:@"virtual"].multiArrayValue mutable:(MLMultiArray *)[prediction featureValueForName:@"mutable"].multiArrayValue explicit:(MLMultiArray *)[prediction featureValueForName:@"explicit"].multiArrayValue export:(MLMultiArray *)[prediction featureValueForName:@"export"].multiArrayValue Any:(MLMultiArray *)[prediction featureValueForName:@"Any"].multiArrayValue Self_:(MLMultiArray *)[prediction featureValueForName:@"Self"].multiArrayValue some:(MLMultiArray *)[prediction featureValueForName:@"some"].multiArrayValue any_workaround:(MLMultiArray *)[prediction featureValueForName:@"any_workaround"].multiArrayValue do_:(MLMultiArray *)[prediction featureValueForName:@"do"].multiArrayValue else_:(MLMultiArray *)[prediction featureValueForName:@"else"].multiArrayValue open:(MLMultiArray *)[prediction featureValueForName:@"open"].multiArrayValue fileprivate:(MLMultiArray *)[prediction featureValueForName:@"fileprivate"].multiArrayValue convenience_:(MLMultiArray *)[prediction featureValueForName:@"convenience"].multiArrayValue dynamic_:(MLMultiArray *)[prediction featureValueForName:@"dynamic"].multiArrayValue final_:(MLMultiArray *)[prediction featureValueForName:@"final"].multiArrayValue lazy_:(MLMultiArray *)[prediction featureValueForName:@"lazy"].multiArrayValue optional_:(MLMultiArray *)[prediction featureValueForName:@"optional"].multiArrayValue required_:(MLMultiArray *)[prediction featureValueForName:@"required"].multiArrayValue override_:(MLMultiArray *)[prediction featureValueForName:@"override"].multiArrayValue weak_:(MLMultiArray *)[prediction featureValueForName:@"weak"].multiArrayValue unowned_:(MLMultiArray *)[prediction featureValueForName:@"unowned"].multiArrayValue mutating_:(MLMultiArray *)[prediction featureValueForName:@"mutating"].multiArrayValue nonmutating_:(MLMultiArray *)[prediction featureValueForName:@"nonmutating"].multiArrayValue indirect:(MLMultiArray *)[prediction featureValueForName:@"indirect"].multiArrayValue prefix_:(MLMultiArray *)[prediction featureValueForName:@"prefix"].multiArrayValue postfix_:(MLMultiArray *)[prediction featureValueForName:@"postfix"].multiArrayValue infix_:(MLMultiArray *)[prediction featureValueForName:@"infix"].multiArrayValue get_:(MLMultiArray *)[prediction featureValueForName:@"get"].multiArrayValue set_:(MLMultiArray *)[prediction featureValueForName:@"set"].multiArrayValue willSet:(MLMultiArray *)[prediction featureValueForName:@"willSet"].multiArrayValue didSet_:(MLMultiArray *)[prediction featureValueForName:@"didSet"].multiArrayValue NULL_:(MLMultiArray *)[prediction featureValueForName:@"NULL"].multiArrayValue Class_:(MLMultiArray *)[prediction featureValueForName:@"Class"].multiArrayValue SEL_:(MLMultiArray *)[prediction featureValueForName:@"SEL"].multiArrayValue IMP_:(MLMultiArray *)[prediction featureValueForName:@"IMP"].multiArrayValue nonatomic_:(MLMultiArray *)[prediction featureValueForName:@"nonatomic"].multiArrayValue atomic_:(MLMultiArray *)[prediction featureValueForName:@"atomic"].multiArrayValue strong:(MLMultiArray *)[prediction featureValueForName:@"strong"].multiArrayValue copy:(MLMultiArray *)[prediction featureValueForName:@"copy"].multiArrayValue assign:(MLMultiArray *)[prediction featureValueForName:@"assign"].multiArrayValue readonly:(MLMultiArray *)[prediction featureValueForName:@"readonly"].multiArrayValue readwrite:(MLMultiArray *)[prediction featureValueForName:@"readwrite"].multiArrayValue instancetype:(MLMultiArray *)[prediction featureValueForName:@"instancetype"].multiArrayValue description:(MLMultiArray *)[prediction featureValueForName:@"description"].multiArrayValue hash:(MLMultiArray *)[prediction featureValueForName:@"hash"].multiArrayValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (void)predictionFromFeatures:(NamesInput *)input options:(MLPredictionOptions *)options completionHandler:(void (^)(NamesOutput * _Nullable output, NSError * _Nullable error))completionHandler {
    [self.model predictionFromFeatures:input options:options completionHandler:^(id<MLFeatureProvider> prediction, NSError *predictionError) {
        if (prediction != nil) {
            NamesOutput *output = [[NamesOutput alloc] initWithOut:(MLMultiArray *)[prediction featureValueForName:@"out"].multiArrayValue var_:(MLMultiArray *)[prediction featureValueForName:@"var"].multiArrayValue in_:(MLMultiArray *)[prediction featureValueForName:@"in"].multiArrayValue inout_:(MLMultiArray *)[prediction featureValueForName:@"inout"].multiArrayValue self_:(MLMultiArray *)[prediction featureValueForName:@"self"].multiArrayValue class_:(MLMultiArray *)[prediction featureValueForName:@"class"].multiArrayValue func_workaround:(MLMultiArray *)[prediction featureValueForName:@"func_workaround"].multiArrayValue let_:(MLMultiArray *)[prediction featureValueForName:@"let"].multiArrayValue import_:(MLMultiArray *)[prediction featureValueForName:@"import"].multiArrayValue return_:(MLMultiArray *)[prediction featureValueForName:@"return"].multiArrayValue if_:(MLMultiArray *)[prediction featureValueForName:@"if"].multiArrayValue for_:(MLMultiArray *)[prediction featureValueForName:@"for"].multiArrayValue while_:(MLMultiArray *)[prediction featureValueForName:@"while"].multiArrayValue struct_:(MLMultiArray *)[prediction featureValueForName:@"struct"].multiArrayValue protocol_:(MLMultiArray *)[prediction featureValueForName:@"protocol"].multiArrayValue model:(MLMultiArray *)[prediction featureValueForName:@"model"].multiArrayValue input:(MLMultiArray *)[prediction featureValueForName:@"input"].multiArrayValue output:(MLMultiArray *)[prediction featureValueForName:@"output"].multiArrayValue features:(MLMultiArray *)[prediction featureValueForName:@"features"].multiArrayValue featureNames:(MLMultiArray *)[prediction featureValueForName:@"featureNames"].multiArrayValue provider:(MLMultiArray *)[prediction featureValueForName:@"provider"].multiArrayValue id_:(MLMultiArray *)[prediction featureValueForName:@"id"].multiArrayValue int_:(MLMultiArray *)[prediction featureValueForName:@"int"].multiArrayValue float_:(MLMultiArray *)[prediction featureValueForName:@"float"].multiArrayValue double_:(MLMultiArray *)[prediction featureValueForName:@"double"].multiArrayValue char_:(MLMultiArray *)[prediction featureValueForName:@"char"].multiArrayValue void_:(MLMultiArray *)[prediction featureValueForName:@"void"].multiArrayValue bool_workaround:(MLMultiArray *)[prediction featureValueForName:@"bool_workaround"].multiArrayValue BOOL_:(MLMultiArray *)[prediction featureValueForName:@"BOOL"].multiArrayValue YES_:(MLMultiArray *)[prediction featureValueForName:@"YES"].multiArrayValue NO_:(MLMultiArray *)[prediction featureValueForName:@"NO"].multiArrayValue nil_:(MLMultiArray *)[prediction featureValueForName:@"nil"].multiArrayValue new:(MLMultiArray *)[prediction featureValueForName:@"new"].multiArrayValue default_:(MLMultiArray *)[prediction featureValueForName:@"default"].multiArrayValue switch_:(MLMultiArray *)[prediction featureValueForName:@"switch"].multiArrayValue case_:(MLMultiArray *)[prediction featureValueForName:@"case"].multiArrayValue operator_:(MLMultiArray *)[prediction featureValueForName:@"operator"].multiArrayValue typealias_:(MLMultiArray *)[prediction featureValueForName:@"typealias"].multiArrayValue extension_:(MLMultiArray *)[prediction featureValueForName:@"extension"].multiArrayValue init_:(MLMultiArray *)[prediction featureValueForName:@"init"].multiArrayValue deinit_:(MLMultiArray *)[prediction featureValueForName:@"deinit"].multiArrayValue subscript_:(MLMultiArray *)[prediction featureValueForName:@"subscript"].multiArrayValue static_:(MLMultiArray *)[prediction featureValueForName:@"static"].multiArrayValue public_:(MLMultiArray *)[prediction featureValueForName:@"public"].multiArrayValue private_:(MLMultiArray *)[prediction featureValueForName:@"private"].multiArrayValue internal_:(MLMultiArray *)[prediction featureValueForName:@"internal"].multiArrayValue true_workaround:(MLMultiArray *)[prediction featureValueForName:@"true_workaround"].multiArrayValue false_workaround:(MLMultiArray *)[prediction featureValueForName:@"false_workaround"].multiArrayValue Type_:(MLMultiArray *)[prediction featureValueForName:@"Type"].multiArrayValue Protocol_:(MLMultiArray *)[prediction featureValueForName:@"Protocol"].multiArrayValue super_:(MLMultiArray *)[prediction featureValueForName:@"super"].multiArrayValue where_:(MLMultiArray *)[prediction featureValueForName:@"where"].multiArrayValue as_:(MLMultiArray *)[prediction featureValueForName:@"as"].multiArrayValue is_:(MLMultiArray *)[prediction featureValueForName:@"is"].multiArrayValue try:(MLMultiArray *)[prediction featureValueForName:@"try"].multiArrayValue throw:(MLMultiArray *)[prediction featureValueForName:@"throw"].multiArrayValue throws:(MLMultiArray *)[prediction featureValueForName:@"throws"].multiArrayValue rethrows:(MLMultiArray *)[prediction featureValueForName:@"rethrows"].multiArrayValue async:(MLMultiArray *)[prediction featureValueForName:@"async"].multiArrayValue await:(MLMultiArray *)[prediction featureValueForName:@"await"].multiArrayValue defer:(MLMultiArray *)[prediction featureValueForName:@"defer"].multiArrayValue guard:(MLMultiArray *)[prediction featureValueForName:@"guard"].multiArrayValue repeat:(MLMultiArray *)[prediction featureValueForName:@"repeat"].multiArrayValue break_:(MLMultiArray *)[prediction featureValueForName:@"break"].multiArrayValue continue_:(MLMultiArray *)[prediction featureValueForName:@"continue"].multiArrayValue fallthrough_:(MLMultiArray *)[prediction featureValueForName:@"fallthrough"].multiArrayValue associatedtype:(MLMultiArray *)[prediction featureValueForName:@"associatedtype"].multiArrayValue enum_:(MLMultiArray *)[prediction featureValueForName:@"enum"].multiArrayValue union_:(MLMultiArray *)[prediction featureValueForName:@"union"].multiArrayValue const_:(MLMultiArray *)[prediction featureValueForName:@"const"].multiArrayValue signed_:(MLMultiArray *)[prediction featureValueForName:@"signed"].multiArrayValue unsigned_:(MLMultiArray *)[prediction featureValueForName:@"unsigned"].multiArrayValue long_:(MLMultiArray *)[prediction featureValueForName:@"long"].multiArrayValue short_:(MLMultiArray *)[prediction featureValueForName:@"short"].multiArrayValue auto_:(MLMultiArray *)[prediction featureValueForName:@"auto"].multiArrayValue register_:(MLMultiArray *)[prediction featureValueForName:@"register"].multiArrayValue extern_:(MLMultiArray *)[prediction featureValueForName:@"extern"].multiArrayValue volatile_:(MLMultiArray *)[prediction featureValueForName:@"volatile"].multiArrayValue goto_:(MLMultiArray *)[prediction featureValueForName:@"goto"].multiArrayValue sizeof_:(MLMultiArray *)[prediction featureValueForName:@"sizeof"].multiArrayValue typedef_:(MLMultiArray *)[prediction featureValueForName:@"typedef"].multiArrayValue oneway_:(MLMultiArray *)[prediction featureValueForName:@"oneway"].multiArrayValue bycopy_:(MLMultiArray *)[prediction featureValueForName:@"bycopy"].multiArrayValue byref_:(MLMultiArray *)[prediction featureValueForName:@"byref"].multiArrayValue restrict_:(MLMultiArray *)[prediction featureValueForName:@"restrict"].multiArrayValue inline_:(MLMultiArray *)[prediction featureValueForName:@"inline"].multiArrayValue template:(MLMultiArray *)[prediction featureValueForName:@"template"].multiArrayValue namespace:(MLMultiArray *)[prediction featureValueForName:@"namespace"].multiArrayValue delete:(MLMultiArray *)[prediction featureValueForName:@"delete"].multiArrayValue this:(MLMultiArray *)[prediction featureValueForName:@"this"].multiArrayValue friend:(MLMultiArray *)[prediction featureValueForName:@"friend"].multiArrayValue virtual:(MLMultiArray *)[prediction featureValueForName:@"virtual"].multiArrayValue mutable:(MLMultiArray *)[prediction featureValueForName:@"mutable"].multiArrayValue explicit:(MLMultiArray *)[prediction featureValueForName:@"explicit"].multiArrayValue export:(MLMultiArray *)[prediction featureValueForName:@"export"].multiArrayValue Any:(MLMultiArray *)[prediction featureValueForName:@"Any"].multiArrayValue Self_:(MLMultiArray *)[prediction featureValueForName:@"Self"].multiArrayValue some:(MLMultiArray *)[prediction featureValueForName:@"some"].multiArrayValue any_workaround:(MLMultiArray *)[prediction featureValueForName:@"any_workaround"].multiArrayValue do_:(MLMultiArray *)[prediction featureValueForName:@"do"].multiArrayValue else_:(MLMultiArray *)[prediction featureValueForName:@"else"].multiArrayValue open:(MLMultiArray *)[prediction featureValueForName:@"open"].multiArrayValue fileprivate:(MLMultiArray *)[prediction featureValueForName:@"fileprivate"].multiArrayValue convenience_:(MLMultiArray *)[prediction featureValueForName:@"convenience"].multiArrayValue dynamic_:(MLMultiArray *)[prediction featureValueForName:@"dynamic"].multiArrayValue final_:(MLMultiArray *)[prediction featureValueForName:@"final"].multiArrayValue lazy_:(MLMultiArray *)[prediction featureValueForName:@"lazy"].multiArrayValue optional_:(MLMultiArray *)[prediction featureValueForName:@"optional"].multiArrayValue required_:(MLMultiArray *)[prediction featureValueForName:@"required"].multiArrayValue override_:(MLMultiArray *)[prediction featureValueForName:@"override"].multiArrayValue weak_:(MLMultiArray *)[prediction featureValueForName:@"weak"].multiArrayValue unowned_:(MLMultiArray *)[prediction featureValueForName:@"unowned"].multiArrayValue mutating_:(MLMultiArray *)[prediction featureValueForName:@"mutating"].multiArrayValue nonmutating_:(MLMultiArray *)[prediction featureValueForName:@"nonmutating"].multiArrayValue indirect:(MLMultiArray *)[prediction featureValueForName:@"indirect"].multiArrayValue prefix_:(MLMultiArray *)[prediction featureValueForName:@"prefix"].multiArrayValue postfix_:(MLMultiArray *)[prediction featureValueForName:@"postfix"].multiArrayValue infix_:(MLMultiArray *)[prediction featureValueForName:@"infix"].multiArrayValue get_:(MLMultiArray *)[prediction featureValueForName:@"get"].multiArrayValue set_:(MLMultiArray *)[prediction featureValueForName:@"set"].multiArrayValue willSet:(MLMultiArray *)[prediction featureValueForName:@"willSet"].multiArrayValue didSet_:(MLMultiArray *)[prediction featureValueForName:@"didSet"].multiArrayValue NULL_:(MLMultiArray *)[prediction featureValueForName:@"NULL"].multiArrayValue Class_:(MLMultiArray *)[prediction featureValueForName:@"Class"].multiArrayValue SEL_:(MLMultiArray *)[prediction featureValueForName:@"SEL"].multiArrayValue IMP_:(MLMultiArray *)[prediction featureValueForName:@"IMP"].multiArrayValue nonatomic_:(MLMultiArray *)[prediction featureValueForName:@"nonatomic"].multiArrayValue atomic_:(MLMultiArray *)[prediction featureValueForName:@"atomic"].multiArrayValue strong:(MLMultiArray *)[prediction featureValueForName:@"strong"].multiArrayValue copy:(MLMultiArray *)[prediction featureValueForName:@"copy"].multiArrayValue assign:(MLMultiArray *)[prediction featureValueForName:@"assign"].multiArrayValue readonly:(MLMultiArray *)[prediction featureValueForName:@"readonly"].multiArrayValue readwrite:(MLMultiArray *)[prediction featureValueForName:@"readwrite"].multiArrayValue instancetype:(MLMultiArray *)[prediction featureValueForName:@"instancetype"].multiArrayValue description:(MLMultiArray *)[prediction featureValueForName:@"description"].multiArrayValue hash:(MLMultiArray *)[prediction featureValueForName:@"hash"].multiArrayValue];
            completionHandler(output, predictionError);
        } else {
            completionHandler(nil, predictionError);
        }
    }];
}

- (nullable NamesOutput *)predictionFromX:(MLMultiArray *)x error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    NamesInput *input_ = [[NamesInput alloc] initWithX:x];
    return [self predictionFromFeatures:input_ error:error];
}

- (nullable NSArray<NamesOutput *> *)predictionsFromInputs:(NSArray<NamesInput*> *)inputArray options:(MLPredictionOptions *)options error:(NSError * _Nullable __autoreleasing * _Nullable)error {
    id<MLBatchProvider> inBatch = [[MLArrayBatchProvider alloc] initWithFeatureProviderArray:inputArray];
    id<MLBatchProvider> outBatch = [self.model predictionsFromBatch:inBatch options:options error:error];
    if (!outBatch) { return nil; }
    NSMutableArray<NamesOutput*> *results = [NSMutableArray arrayWithCapacity:(NSUInteger)outBatch.count];
    for (NSInteger i = 0; i < outBatch.count; i++) {
        id<MLFeatureProvider> resultProvider = [outBatch featuresAtIndex:i];
        NamesOutput * result = [[NamesOutput alloc] initWithOut:(MLMultiArray *)[resultProvider featureValueForName:@"out"].multiArrayValue var_:(MLMultiArray *)[resultProvider featureValueForName:@"var"].multiArrayValue in_:(MLMultiArray *)[resultProvider featureValueForName:@"in"].multiArrayValue inout_:(MLMultiArray *)[resultProvider featureValueForName:@"inout"].multiArrayValue self_:(MLMultiArray *)[resultProvider featureValueForName:@"self"].multiArrayValue class_:(MLMultiArray *)[resultProvider featureValueForName:@"class"].multiArrayValue func_workaround:(MLMultiArray *)[resultProvider featureValueForName:@"func_workaround"].multiArrayValue let_:(MLMultiArray *)[resultProvider featureValueForName:@"let"].multiArrayValue import_:(MLMultiArray *)[resultProvider featureValueForName:@"import"].multiArrayValue return_:(MLMultiArray *)[resultProvider featureValueForName:@"return"].multiArrayValue if_:(MLMultiArray *)[resultProvider featureValueForName:@"if"].multiArrayValue for_:(MLMultiArray *)[resultProvider featureValueForName:@"for"].multiArrayValue while_:(MLMultiArray *)[resultProvider featureValueForName:@"while"].multiArrayValue struct_:(MLMultiArray *)[resultProvider featureValueForName:@"struct"].multiArrayValue protocol_:(MLMultiArray *)[resultProvider featureValueForName:@"protocol"].multiArrayValue model:(MLMultiArray *)[resultProvider featureValueForName:@"model"].multiArrayValue input:(MLMultiArray *)[resultProvider featureValueForName:@"input"].multiArrayValue output:(MLMultiArray *)[resultProvider featureValueForName:@"output"].multiArrayValue features:(MLMultiArray *)[resultProvider featureValueForName:@"features"].multiArrayValue featureNames:(MLMultiArray *)[resultProvider featureValueForName:@"featureNames"].multiArrayValue provider:(MLMultiArray *)[resultProvider featureValueForName:@"provider"].multiArrayValue id_:(MLMultiArray *)[resultProvider featureValueForName:@"id"].multiArrayValue int_:(MLMultiArray *)[resultProvider featureValueForName:@"int"].multiArrayValue float_:(MLMultiArray *)[resultProvider featureValueForName:@"float"].multiArrayValue double_:(MLMultiArray *)[resultProvider featureValueForName:@"double"].multiArrayValue char_:(MLMultiArray *)[resultProvider featureValueForName:@"char"].multiArrayValue void_:(MLMultiArray *)[resultProvider featureValueForName:@"void"].multiArrayValue bool_workaround:(MLMultiArray *)[resultProvider featureValueForName:@"bool_workaround"].multiArrayValue BOOL_:(MLMultiArray *)[resultProvider featureValueForName:@"BOOL"].multiArrayValue YES_:(MLMultiArray *)[resultProvider featureValueForName:@"YES"].multiArrayValue NO_:(MLMultiArray *)[resultProvider featureValueForName:@"NO"].multiArrayValue nil_:(MLMultiArray *)[resultProvider featureValueForName:@"nil"].multiArrayValue new:(MLMultiArray *)[resultProvider featureValueForName:@"new"].multiArrayValue default_:(MLMultiArray *)[resultProvider featureValueForName:@"default"].multiArrayValue switch_:(MLMultiArray *)[resultProvider featureValueForName:@"switch"].multiArrayValue case_:(MLMultiArray *)[resultProvider featureValueForName:@"case"].multiArrayValue operator_:(MLMultiArray *)[resultProvider featureValueForName:@"operator"].multiArrayValue typealias_:(MLMultiArray *)[resultProvider featureValueForName:@"typealias"].multiArrayValue extension_:(MLMultiArray *)[resultProvider featureValueForName:@"extension"].multiArrayValue init_:(MLMultiArray *)[resultProvider featureValueForName:@"init"].multiArrayValue deinit_:(MLMultiArray *)[resultProvider featureValueForName:@"deinit"].multiArrayValue subscript_:(MLMultiArray *)[resultProvider featureValueForName:@"subscript"].multiArrayValue static_:(MLMultiArray *)[resultProvider featureValueForName:@"static"].multiArrayValue public_:(MLMultiArray *)[resultProvider featureValueForName:@"public"].multiArrayValue private_:(MLMultiArray *)[resultProvider featureValueForName:@"private"].multiArrayValue internal_:(MLMultiArray *)[resultProvider featureValueForName:@"internal"].multiArrayValue true_workaround:(MLMultiArray *)[resultProvider featureValueForName:@"true_workaround"].multiArrayValue false_workaround:(MLMultiArray *)[resultProvider featureValueForName:@"false_workaround"].multiArrayValue Type_:(MLMultiArray *)[resultProvider featureValueForName:@"Type"].multiArrayValue Protocol_:(MLMultiArray *)[resultProvider featureValueForName:@"Protocol"].multiArrayValue super_:(MLMultiArray *)[resultProvider featureValueForName:@"super"].multiArrayValue where_:(MLMultiArray *)[resultProvider featureValueForName:@"where"].multiArrayValue as_:(MLMultiArray *)[resultProvider featureValueForName:@"as"].multiArrayValue is_:(MLMultiArray *)[resultProvider featureValueForName:@"is"].multiArrayValue try:(MLMultiArray *)[resultProvider featureValueForName:@"try"].multiArrayValue throw:(MLMultiArray *)[resultProvider featureValueForName:@"throw"].multiArrayValue throws:(MLMultiArray *)[resultProvider featureValueForName:@"throws"].multiArrayValue rethrows:(MLMultiArray *)[resultProvider featureValueForName:@"rethrows"].multiArrayValue async:(MLMultiArray *)[resultProvider featureValueForName:@"async"].multiArrayValue await:(MLMultiArray *)[resultProvider featureValueForName:@"await"].multiArrayValue defer:(MLMultiArray *)[resultProvider featureValueForName:@"defer"].multiArrayValue guard:(MLMultiArray *)[resultProvider featureValueForName:@"guard"].multiArrayValue repeat:(MLMultiArray *)[resultProvider featureValueForName:@"repeat"].multiArrayValue break_:(MLMultiArray *)[resultProvider featureValueForName:@"break"].multiArrayValue continue_:(MLMultiArray *)[resultProvider featureValueForName:@"continue"].multiArrayValue fallthrough_:(MLMultiArray *)[resultProvider featureValueForName:@"fallthrough"].multiArrayValue associatedtype:(MLMultiArray *)[resultProvider featureValueForName:@"associatedtype"].multiArrayValue enum_:(MLMultiArray *)[resultProvider featureValueForName:@"enum"].multiArrayValue union_:(MLMultiArray *)[resultProvider featureValueForName:@"union"].multiArrayValue const_:(MLMultiArray *)[resultProvider featureValueForName:@"const"].multiArrayValue signed_:(MLMultiArray *)[resultProvider featureValueForName:@"signed"].multiArrayValue unsigned_:(MLMultiArray *)[resultProvider featureValueForName:@"unsigned"].multiArrayValue long_:(MLMultiArray *)[resultProvider featureValueForName:@"long"].multiArrayValue short_:(MLMultiArray *)[resultProvider featureValueForName:@"short"].multiArrayValue auto_:(MLMultiArray *)[resultProvider featureValueForName:@"auto"].multiArrayValue register_:(MLMultiArray *)[resultProvider featureValueForName:@"register"].multiArrayValue extern_:(MLMultiArray *)[resultProvider featureValueForName:@"extern"].multiArrayValue volatile_:(MLMultiArray *)[resultProvider featureValueForName:@"volatile"].multiArrayValue goto_:(MLMultiArray *)[resultProvider featureValueForName:@"goto"].multiArrayValue sizeof_:(MLMultiArray *)[resultProvider featureValueForName:@"sizeof"].multiArrayValue typedef_:(MLMultiArray *)[resultProvider featureValueForName:@"typedef"].multiArrayValue oneway_:(MLMultiArray *)[resultProvider featureValueForName:@"oneway"].multiArrayValue bycopy_:(MLMultiArray *)[resultProvider featureValueForName:@"bycopy"].multiArrayValue byref_:(MLMultiArray *)[resultProvider featureValueForName:@"byref"].multiArrayValue restrict_:(MLMultiArray *)[resultProvider featureValueForName:@"restrict"].multiArrayValue inline_:(MLMultiArray *)[resultProvider featureValueForName:@"inline"].multiArrayValue template:(MLMultiArray *)[resultProvider featureValueForName:@"template"].multiArrayValue namespace:(MLMultiArray *)[resultProvider featureValueForName:@"namespace"].multiArrayValue delete:(MLMultiArray *)[resultProvider featureValueForName:@"delete"].multiArrayValue this:(MLMultiArray *)[resultProvider featureValueForName:@"this"].multiArrayValue friend:(MLMultiArray *)[resultProvider featureValueForName:@"friend"].multiArrayValue virtual:(MLMultiArray *)[resultProvider featureValueForName:@"virtual"].multiArrayValue mutable:(MLMultiArray *)[resultProvider featureValueForName:@"mutable"].multiArrayValue explicit:(MLMultiArray *)[resultProvider featureValueForName:@"explicit"].multiArrayValue export:(MLMultiArray *)[resultProvider featureValueForName:@"export"].multiArrayValue Any:(MLMultiArray *)[resultProvider featureValueForName:@"Any"].multiArrayValue Self_:(MLMultiArray *)[resultProvider featureValueForName:@"Self"].multiArrayValue some:(MLMultiArray *)[resultProvider featureValueForName:@"some"].multiArrayValue any_workaround:(MLMultiArray *)[resultProvider featureValueForName:@"any_workaround"].multiArrayValue do_:(MLMultiArray *)[resultProvider featureValueForName:@"do"].multiArrayValue else_:(MLMultiArray *)[resultProvider featureValueForName:@"else"].multiArrayValue open:(MLMultiArray *)[resultProvider featureValueForName:@"open"].multiArrayValue fileprivate:(MLMultiArray *)[resultProvider featureValueForName:@"fileprivate"].multiArrayValue convenience_:(MLMultiArray *)[resultProvider featureValueForName:@"convenience"].multiArrayValue dynamic_:(MLMultiArray *)[resultProvider featureValueForName:@"dynamic"].multiArrayValue final_:(MLMultiArray *)[resultProvider featureValueForName:@"final"].multiArrayValue lazy_:(MLMultiArray *)[resultProvider featureValueForName:@"lazy"].multiArrayValue optional_:(MLMultiArray *)[resultProvider featureValueForName:@"optional"].multiArrayValue required_:(MLMultiArray *)[resultProvider featureValueForName:@"required"].multiArrayValue override_:(MLMultiArray *)[resultProvider featureValueForName:@"override"].multiArrayValue weak_:(MLMultiArray *)[resultProvider featureValueForName:@"weak"].multiArrayValue unowned_:(MLMultiArray *)[resultProvider featureValueForName:@"unowned"].multiArrayValue mutating_:(MLMultiArray *)[resultProvider featureValueForName:@"mutating"].multiArrayValue nonmutating_:(MLMultiArray *)[resultProvider featureValueForName:@"nonmutating"].multiArrayValue indirect:(MLMultiArray *)[resultProvider featureValueForName:@"indirect"].multiArrayValue prefix_:(MLMultiArray *)[resultProvider featureValueForName:@"prefix"].multiArrayValue postfix_:(MLMultiArray *)[resultProvider featureValueForName:@"postfix"].multiArrayValue infix_:(MLMultiArray *)[resultProvider featureValueForName:@"infix"].multiArrayValue get_:(MLMultiArray *)[resultProvider featureValueForName:@"get"].multiArrayValue set_:(MLMultiArray *)[resultProvider featureValueForName:@"set"].multiArrayValue willSet:(MLMultiArray *)[resultProvider featureValueForName:@"willSet"].multiArrayValue didSet_:(MLMultiArray *)[resultProvider featureValueForName:@"didSet"].multiArrayValue NULL_:(MLMultiArray *)[resultProvider featureValueForName:@"NULL"].multiArrayValue Class_:(MLMultiArray *)[resultProvider featureValueForName:@"Class"].multiArrayValue SEL_:(MLMultiArray *)[resultProvider featureValueForName:@"SEL"].multiArrayValue IMP_:(MLMultiArray *)[resultProvider featureValueForName:@"IMP"].multiArrayValue nonatomic_:(MLMultiArray *)[resultProvider featureValueForName:@"nonatomic"].multiArrayValue atomic_:(MLMultiArray *)[resultProvider featureValueForName:@"atomic"].multiArrayValue strong:(MLMultiArray *)[resultProvider featureValueForName:@"strong"].multiArrayValue copy:(MLMultiArray *)[resultProvider featureValueForName:@"copy"].multiArrayValue assign:(MLMultiArray *)[resultProvider featureValueForName:@"assign"].multiArrayValue readonly:(MLMultiArray *)[resultProvider featureValueForName:@"readonly"].multiArrayValue readwrite:(MLMultiArray *)[resultProvider featureValueForName:@"readwrite"].multiArrayValue instancetype:(MLMultiArray *)[resultProvider featureValueForName:@"instancetype"].multiArrayValue description:(MLMultiArray *)[resultProvider featureValueForName:@"description"].multiArrayValue hash:(MLMultiArray *)[resultProvider featureValueForName:@"hash"].multiArrayValue];
        [results addObject:result];
    }
    return results;
}

@end
