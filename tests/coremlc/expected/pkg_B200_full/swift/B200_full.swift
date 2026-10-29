//
// B200_full.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 10.15, iOS 13.0, tvOS 13.0, watchOS 6.0, visionOS 1.0, *)
public class B200_fullInput : MLFeatureProvider {

    /// orig label0 as 1 element vector of doubles
    public var label0: MLMultiArray

    public var featureNames: Set<String> { ["label0"] }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        if featureName == "label0" {
            return MLFeatureValue(multiArray: label0)
        }
        return nil
    }

    public init(label0: MLMultiArray) {
        self.label0 = label0
    }

    @available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
    public convenience init(label0: MLShapedArray<Double>) {
        self.init(label0: MLMultiArray(label0))
    }

}


/// Model Prediction Output Type
@available(macOS 10.15, iOS 13.0, tvOS 13.0, watchOS 6.0, visionOS 1.0, *)
public class B200_fullOutput : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// orig z as integer value
    public var z: Int64 {
        provider.featureValue(for: "z")!.int64Value
    }

    /// orig x as dictionary of 64-bit integers to doubles
    public var x: [Int64 : Double] {
        provider.featureValue(for: "x")!.dictionaryValue as! [Int64 : Double]
    }

    public var featureNames: Set<String> {
        provider.featureNames
    }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        provider.featureValue(for: featureName)
    }

    public init(z: Int64, x: [Int64 : Double]) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["z" : MLFeatureValue(int64: z), "x" : MLFeatureValue(dictionary: x as [AnyHashable : NSNumber])])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}


/// Class for model loading and prediction
@available(macOS 10.15, iOS 13.0, tvOS 13.0, watchOS 6.0, visionOS 1.0, *)
public class B200_full {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "B200_full", withExtension:"mlmodelc")!
    }

    /**
        Construct B200_full instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of B200_full.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `B200_full.urlOfModelInThisBundle` to create a MLModel object to pass-in.

        - parameters:
          - model: MLModel object
    */
    init(model: MLModel) {
        self.model = model
    }

    /**
        Construct B200_full instance by automatically loading the model from the app's bundle.
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
        Construct B200_full instance with explicit path to mlmodelc file
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
        Construct B200_full instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    @available(macOS 11.0, iOS 14.0, tvOS 14.0, watchOS 7.0, visionOS 1.0, *)
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<B200_full, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct B200_full instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    @available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> B200_full {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct B200_full instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    @available(macOS 11.0, iOS 14.0, tvOS 14.0, watchOS 7.0, visionOS 1.0, *)
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<B200_full, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(B200_full(model: model)))
            }
        }
    }

    /**
        Construct B200_full instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    @available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> B200_full {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return B200_full(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as B200_fullInput

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as B200_fullOutput
    */
    public func prediction(input: B200_fullInput) throws -> B200_fullOutput {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as B200_fullInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as B200_fullOutput
    */
    public func prediction(input: B200_fullInput, options: MLPredictionOptions) throws -> B200_fullOutput {
        let outFeatures = try model.prediction(from: input, options: options)
        return B200_fullOutput(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as B200_fullInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as B200_fullOutput
    */
    @available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
    public func prediction(input: B200_fullInput, options: MLPredictionOptions = MLPredictionOptions()) async throws -> B200_fullOutput {
        let outFeatures = try await model.prediction(from: input, options: options)
        return B200_fullOutput(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - label0: orig label0 as 1 element vector of doubles

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as B200_fullOutput
    */
    public func prediction(label0: MLMultiArray) throws -> B200_fullOutput {
        let input_ = B200_fullInput(label0: label0)
        return try prediction(input: input_)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - label0: orig label0 as 1 element vector of doubles

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as B200_fullOutput
    */

    @available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
    public func prediction(label0: MLShapedArray<Double>) throws -> B200_fullOutput {
        let input_ = B200_fullInput(label0: label0)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [B200_fullInput]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [B200_fullOutput]
    */
    public func predictions(inputs: [B200_fullInput], options: MLPredictionOptions = MLPredictionOptions()) throws -> [B200_fullOutput] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [B200_fullOutput] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  B200_fullOutput(features: outProvider)
            results.append(result)
        }
        return results
    }
}
