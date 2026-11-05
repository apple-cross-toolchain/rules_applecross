//
//  Thing+CoreDataProperties.swift
//  
//
//  Created by momc.
//
//  This file was automatically generated and should not be edited.
//

public import Foundation
public import CoreData


public typealias ThingCoreDataPropertiesSet = NSSet

extension Thing {

    @nonobjc public class func fetchRequest() -> NSFetchRequest<Thing> {
        return NSFetchRequest<Thing>(entityName: "Thing")
    }

    @NSManaged nonisolated public var s: String?
    @NSManaged nonisolated public var i16: Int16
    @NSManaged nonisolated public var i32: Int32
    @NSManaged nonisolated public var i64: Int64
    @NSManaged nonisolated public var dbl: Double
    @NSManaged nonisolated public var flt: Float
    @NSManaged nonisolated public var b: Bool
    @NSManaged nonisolated public var dec: NSDecimalNumber?
    @NSManaged nonisolated public var date: Date?
    @NSManaged nonisolated public var data: Data?
    @NSManaged nonisolated public var uuid: UUID?
    @NSManaged nonisolated public var uri: URL?
    @NSManaged nonisolated public var obj: NSNumber?
    @NSManaged nonisolated public var req: String?
    @NSManaged nonisolated public var reqi: Int32
    @NSManaged nonisolated public var t: [String]?
    @NSManaged nonisolated public var t2: NSObject?

}

extension Thing : Identifiable {

}
