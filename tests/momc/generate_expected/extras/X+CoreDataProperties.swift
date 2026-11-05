//
//  X+CoreDataProperties.swift
//  
//
//  Created by momc.
//
//  This file was automatically generated and should not be edited.
//

public import Foundation
public import CoreData


public typealias XCoreDataPropertiesSet = NSSet

extension X {

    @nonobjc public class func fetchRequest() -> NSFetchRequest<X> {
        return NSFetchRequest<X>(entityName: "X")
    }

    @NSManaged nonisolated public var name: String?
    @NSManaged nonisolated public var up: String?

}

extension X : Identifiable {

}
