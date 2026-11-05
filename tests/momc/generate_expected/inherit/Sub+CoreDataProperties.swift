//
//  Sub+CoreDataProperties.swift
//  
//
//  Created by momc.
//
//  This file was automatically generated and should not be edited.
//

public import Foundation
public import CoreData


public typealias SubCoreDataPropertiesSet = NSSet

extension Sub {

    @nonobjc public class func fetchRequest() -> NSFetchRequest<Sub> {
        return NSFetchRequest<Sub>(entityName: "Sub")
    }

    @NSManaged nonisolated public var b: String?

}
