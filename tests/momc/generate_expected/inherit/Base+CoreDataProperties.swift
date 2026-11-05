//
//  Base+CoreDataProperties.swift
//  
//
//  Created by momc.
//
//  This file was automatically generated and should not be edited.
//

public import Foundation
public import CoreData


public typealias BaseCoreDataPropertiesSet = NSSet

extension Base {

    @nonobjc public class func fetchRequest() -> NSFetchRequest<Base> {
        return NSFetchRequest<Base>(entityName: "Base")
    }

    @NSManaged nonisolated public var a: String?

}

extension Base : Identifiable {

}
