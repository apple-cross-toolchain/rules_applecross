//
//  Sub2+CoreDataProperties.swift
//  
//
//  Created by momc.
//
//  This file was automatically generated and should not be edited.
//

public import Foundation
public import CoreData


public typealias Sub2CoreDataPropertiesSet = NSSet

extension Sub2 {

    @nonobjc public class func fetchRequest() -> NSFetchRequest<Sub2> {
        return NSFetchRequest<Sub2>(entityName: "Sub2")
    }

    @NSManaged nonisolated public var c: String?

}
