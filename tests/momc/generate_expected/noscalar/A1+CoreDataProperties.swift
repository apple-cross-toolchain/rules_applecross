//
//  A1+CoreDataProperties.swift
//  
//
//  Created by momc.
//
//  This file was automatically generated and should not be edited.
//

public import Foundation
public import CoreData


public typealias A1CoreDataPropertiesSet = NSSet

extension A1 {

    @nonobjc public class func fetchRequest() -> NSFetchRequest<A1> {
        return NSFetchRequest<A1>(entityName: "A1")
    }

    @NSManaged nonisolated public var i: Int32
    @NSManaged nonisolated public var b: Bool
    @NSManaged nonisolated public var d: Double
    @NSManaged nonisolated public var t: Date?
    @NSManaged nonisolated public var peer: NSManagedObject?
    @NSManaged nonisolated public var peers: NSSet?

}

// MARK: Generated accessors for peers
extension A1 {

    @objc(addPeersObject:)
    @NSManaged nonisolated public func addToPeers(_ value: NSManagedObject)

    @objc(removePeersObject:)
    @NSManaged nonisolated public func removeFromPeers(_ value: NSManagedObject)

    @objc(addPeers:)
    @NSManaged nonisolated public func addToPeers(_ values: NSSet)

    @objc(removePeers:)
    @NSManaged nonisolated public func removeFromPeers(_ values: NSSet)

}

extension A1 : Identifiable {

}
