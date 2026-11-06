// hashes: the version hashes of Core Data models built with its public API,
// each preceded by a label, so that sha256_log.dylib, inserted into this
// process, shows what Core Data hashes for each (docs/clean-room/momc.md).
#import <CoreData/CoreData.h>

static NSAttributeDescription *attribute(NSString *name, NSAttributeType type) {
    NSAttributeDescription *a = [NSAttributeDescription new];
    a.name = name;
    a.attributeType = type;
    return a;
}

static void show(const char *label, id description) {
    fprintf(stderr, "== %s\n", label);
    (void)[description versionHash];
}

int main(void) {
    @autoreleasepool {
        NSAttributeDescription *a;
        a = attribute(@"t", NSStringAttributeType); a.optional = NO; show("required string", a);
        a = attribute(@"t", NSStringAttributeType); a.transient = YES; show("optional transient", a);
        a = attribute(@"t", NSBinaryDataAttributeType); a.allowsExternalBinaryDataStorage = YES; show("external binary", a);
        a = attribute(@"t", NSStringAttributeType); a.preservesValueInHistoryOnDeletion = YES; show("preserved in history", a);
        a = attribute(@"t", NSBinaryDataAttributeType); a.allowsExternalBinaryDataStorage = YES; a.preservesValueInHistoryOnDeletion = YES; show("both", a);
        a = attribute(@"t", NSStringAttributeType); a.versionHashModifier = @"mod"; show("modifier", a);
        a = attribute(@"é", NSStringAttributeType); show("non-ASCII name", a);
        NSDerivedAttributeDescription *d = [NSDerivedAttributeDescription new];
        d.name = @"d"; d.attributeType = NSInteger64AttributeType; d.derivationExpression = [NSExpression expressionWithFormat:@"title.@count"];
        show("derived", d);

        NSEntityDescription *e = [NSEntityDescription new]; e.name = @"A";
        NSEntityDescription *f = [NSEntityDescription new]; f.name = @"B";
        NSRelationshipDescription *r = [NSRelationshipDescription new];
        r.name = @"rel"; r.destinationEntity = f; r.maxCount = 1; r.optional = YES;
        NSRelationshipDescription *inverse = [NSRelationshipDescription new];
        inverse.name = @"back"; inverse.destinationEntity = e; inverse.maxCount = 0;
        r.inverseRelationship = inverse; inverse.inverseRelationship = r;
        NSRelationshipDescription *ordered = [NSRelationshipDescription new];
        ordered.name = @"many"; ordered.destinationEntity = f; ordered.maxCount = 5; ordered.minCount = 1; ordered.ordered = YES;
        ordered.deleteRule = NSCascadeDeleteRule;
        NSFetchRequest *request = [NSFetchRequest fetchRequestWithEntityName:@"B"]; request.entity = f;
        NSFetchedPropertyDescription *fetched = [NSFetchedPropertyDescription new]; fetched.name = @"fetched"; fetched.fetchRequest = request;
        NSAttributeDescription *transient = attribute(@"scratch", NSStringAttributeType); transient.transient = YES;
        e.properties = @[attribute(@"z", NSStringAttributeType), attribute(@"Zeta", NSInteger16AttributeType), r, ordered, fetched, transient];
        f.properties = @[inverse];
        e.uniquenessConstraints = @[@[@"z", @"Zeta"]];
        e.versionHashModifier = @"em";
        NSEntityDescription *child = [NSEntityDescription new]; child.name = @"C"; child.properties = @[attribute(@"own", NSStringAttributeType)];
        f.subentities = @[child];
        NSManagedObjectModel *m = [NSManagedObjectModel new];
        m.entities = @[e, f, child];
        show("to-one relationship with an inverse", r);
        show("to-many inverse", inverse);
        show("ordered to-many, counts, cascade", ordered);
        show("fetched property", fetched);
        show("entity A: properties out of order, a transient, a constraint, a modifier", e);
        show("entity C, a subentity", child);
        fprintf(stderr, "== the model's checksum\n%s\n", m.versionChecksum.UTF8String);
    }
    return 0;
}
