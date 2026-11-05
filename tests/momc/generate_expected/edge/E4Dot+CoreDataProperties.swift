//
//  E4Dot+CoreDataProperties.swift
//  
//
//  Created by momc.
//
//  This file was automatically generated and should not be edited.
//

public import Foundation
public import CoreData


public typealias E4DotCoreDataPropertiesSet = NSSet

extension E4Dot {

    @nonobjc public class func fetchRequest() -> NSFetchRequest<E4Dot> {
        return NSFetchRequest<E4Dot>(entityName: "E4")
    }

    @NSManaged nonisolated public var z: String?

}

extension E4Dot : Identifiable {

}
