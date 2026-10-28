//
// ItemSim.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 10.15, iOS 13.0, tvOS 13.0, watchOS 6.0, visionOS 1.0, *)
public class ItemSimInput : MLFeatureProvider {

    /// items as dictionary of 64-bit integers to doubles
    public var items: [Int64 : Double]

    /// k as optional integer value
    public var k: Int64? = nil

    public var featureNames: Set<String> { ["items", "k"] }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        if featureName == "items" {
            return try! MLFeatureValue(dictionary: items as [NSObject : NSNumber])
        }
        if featureName == "k" {
            return k == nil ? nil : MLFeatureValue(int64: k!)
        }
        return nil
    }

    public init(items: [Int64 : Double], k: Int64? = nil) {
        self.items = items
        self.k = k
    }

}


/// Model Prediction Output Type
@available(macOS 10.15, iOS 13.0, tvOS 13.0, watchOS 6.0, visionOS 1.0, *)
public class ItemSimOutput : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// recommendations as sequence of 64-bit integers
    public var recommendations: [Int64] {
        provider.featureValue(for: "recommendations")!.sequenceValue!.int64Values as! [Int64]
    }

    public var featureNames: Set<String> {
        provider.featureNames
    }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        provider.featureValue(for: featureName)
    }

    public init(recommendations: [Int64]) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["recommendations" : MLFeatureValue(sequence: MLSequence(int64s: recommendations as [NSNumber]))])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}


/// Class for model loading and prediction
@available(macOS 10.15, iOS 13.0, tvOS 13.0, watchOS 6.0, visionOS 1.0, *)
public class ItemSim {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "ItemSim", withExtension:"mlmodelc")!
    }

    /**
        Construct ItemSim instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of ItemSim.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `ItemSim.urlOfModelInThisBundle` to create a MLModel object to pass-in.

        - parameters:
          - model: MLModel object
    */
    init(model: MLModel) {
        self.model = model
    }

    /**
        Construct ItemSim instance by automatically loading the model from the app's bundle.
    */
    @available(*, deprecated, message: "Use init(configuration:) instead and handle errors appropriately.")
    public convenience init() {
        try! self.init(contentsOf: type(of:self).urlOfModelInThisBundle)
    }

    /**
        Construct a model with configuration

        - parameters:
           - configuration: the desired model configuration

        - throws: an NSError object that describes the problem
    */
    public convenience init(configuration: MLModelConfiguration) throws {
        try self.init(contentsOf: type(of:self).urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct ItemSim instance with explicit path to mlmodelc file
        - parameters:
           - modelURL: the file url of the model

        - throws: an NSError object that describes the problem
    */
    public convenience init(contentsOf modelURL: URL) throws {
        try self.init(model: MLModel(contentsOf: modelURL))
    }

    /**
        Construct a model with URL of the .mlmodelc directory and configuration

        - parameters:
           - modelURL: the file url of the model
           - configuration: the desired model configuration

        - throws: an NSError object that describes the problem
    */
    public convenience init(contentsOf modelURL: URL, configuration: MLModelConfiguration) throws {
        try self.init(model: MLModel(contentsOf: modelURL, configuration: configuration))
    }

    /**
        Construct ItemSim instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    @available(macOS 11.0, iOS 14.0, tvOS 14.0, watchOS 7.0, visionOS 1.0, *)
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<ItemSim, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct ItemSim instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    @available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> ItemSim {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct ItemSim instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    @available(macOS 11.0, iOS 14.0, tvOS 14.0, watchOS 7.0, visionOS 1.0, *)
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<ItemSim, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(ItemSim(model: model)))
            }
        }
    }

    /**
        Construct ItemSim instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    @available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> ItemSim {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return ItemSim(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as ItemSimInput

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as ItemSimOutput
    */
    public func prediction(input: ItemSimInput) throws -> ItemSimOutput {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as ItemSimInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as ItemSimOutput
    */
    public func prediction(input: ItemSimInput, options: MLPredictionOptions) throws -> ItemSimOutput {
        let outFeatures = try model.prediction(from: input, options: options)
        return ItemSimOutput(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as ItemSimInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as ItemSimOutput
    */
    @available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
    public func prediction(input: ItemSimInput, options: MLPredictionOptions = MLPredictionOptions()) async throws -> ItemSimOutput {
        let outFeatures = try await model.prediction(from: input, options: options)
        return ItemSimOutput(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - items: dictionary of 64-bit integers to doubles
            - k: optional integer value

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as ItemSimOutput
    */
    public func prediction(items: [Int64 : Double], k: Int64?) throws -> ItemSimOutput {
        let input_ = ItemSimInput(items: items, k: k)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [ItemSimInput]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [ItemSimOutput]
    */
    public func predictions(inputs: [ItemSimInput], options: MLPredictionOptions = MLPredictionOptions()) throws -> [ItemSimOutput] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [ItemSimOutput] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  ItemSimOutput(features: outProvider)
            results.append(result)
        }
        return results
    }
}
