//
//  E2Model+CoreDataProperties.swift
//  
//
//  Created by momc.
//
//  This file was automatically generated and should not be edited.
//

public import Foundation
public import CoreData


public typealias E2ModelCoreDataPropertiesSet = NSSet

extension E2Model {

    @nonobjc public class func fetchRequest() -> NSFetchRequest<E2Model> {
        return NSFetchRequest<E2Model>(entityName: "E2")
    }

    @NSManaged nonisolated public var x: String?

}

extension E2Model : Identifiable {

}
