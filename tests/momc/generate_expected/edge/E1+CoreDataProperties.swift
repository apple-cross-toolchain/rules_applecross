//
//  E1+CoreDataProperties.swift
//  
//
//  Created by momc.
//
//  This file was automatically generated and should not be edited.
//

public import Foundation
public import CoreData
import App

public typealias E1CoreDataPropertiesSet = NSSet

extension E1 {

    @nonobjc public class func fetchRequest() -> NSFetchRequest<E1> {
        return NSFetchRequest<E1>(entityName: "E1")
    }

    @NSManaged nonisolated public var flag: NSNumber?
    @NSManaged nonisolated public var when: TimeInterval
    @NSManaged nonisolated public var mine: MyType?
    @NSManaged nonisolated public var class: String?
    @NSManaged nonisolated public var d2: NSNumber?
    @NSManaged nonisolated public var owner: E2Model?
    @NSManaged nonisolated public var many: NSSet?

}

// MARK: Generated accessors for many
extension E1 {

    @objc(addManyObject:)
    @NSManaged nonisolated public func addToMany(_ value: E2Model)

    @objc(removeManyObject:)
    @NSManaged nonisolated public func removeFromMany(_ value: E2Model)

    @objc(addMany:)
    @NSManaged nonisolated public func addToMany(_ values: NSSet)

    @objc(removeMany:)
    @NSManaged nonisolated public func removeFromMany(_ values: NSSet)

}

extension E1 : Identifiable {

}
