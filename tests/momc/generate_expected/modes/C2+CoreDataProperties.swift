//
//  C2+CoreDataProperties.swift
//  
//
//  Created by momc.
//
//  This file was automatically generated and should not be edited.
//

public import Foundation
public import CoreData


public typealias C2CoreDataPropertiesSet = NSSet

extension C2 {

    @nonobjc public class func fetchRequest() -> NSFetchRequest<C2> {
        return NSFetchRequest<C2>(entityName: "C2")
    }

    @NSManaged nonisolated public var a: String?

}

extension C2 : Identifiable {

}
