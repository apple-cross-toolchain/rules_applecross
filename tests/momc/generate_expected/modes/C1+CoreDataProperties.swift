//
//  C1+CoreDataProperties.swift
//  
//
//  Created by momc.
//
//  This file was automatically generated and should not be edited.
//

public import Foundation
public import CoreData


public typealias C1CoreDataPropertiesSet = NSSet

extension C1 {

    @nonobjc public class func fetchRequest() -> NSFetchRequest<C1> {
        return NSFetchRequest<C1>(entityName: "C1")
    }

    @NSManaged nonisolated public var a: String?

}

extension C1 : Identifiable {

}
