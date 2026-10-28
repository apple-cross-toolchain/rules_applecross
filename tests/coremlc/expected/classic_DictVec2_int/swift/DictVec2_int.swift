//
// DictVec2_int.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 10.13, iOS 11.0, tvOS 11.0, watchOS 4.0, visionOS 1.0, *)
public class DictVec2_intInput : MLFeatureProvider {

    /// x as dictionary of 64-bit integers to doubles
    public var x: [Int64 : Double]

    public var featureNames: Set<String> { ["x"] }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        if featureName == "x" {
            return try! MLFeatureValue(dictionary: x as [NSObject : NSNumber])
        }
        return nil
    }

    public init(x: [Int64 : Double]) {
        self.x = x
    }

}


/// Model Prediction Output Type
@available(macOS 10.13, iOS 11.0, tvOS 11.0, watchOS 4.0, visionOS 1.0, *)
public class DictVec2_intOutput : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// y as dictionary of 64-bit integers to doubles
    public var y: [Int64 : Double] {
        provider.featureValue(for: "y")!.dictionaryValue as! [Int64 : Double]
    }

    public var featureNames: Set<String> {
        provider.featureNames
    }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        provider.featureValue(for: featureName)
    }

    public init(y: [Int64 : Double]) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["y" : MLFeatureValue(dictionary: y as [AnyHashable : NSNumber])])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}


/// Class for model loading and prediction
@available(macOS 10.13, iOS 11.0, tvOS 11.0, watchOS 4.0, visionOS 1.0, *)
public class DictVec2_int {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "DictVec2_int", withExtension:"mlmodelc")!
    }

    /**
        Construct DictVec2_int instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of DictVec2_int.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `DictVec2_int.urlOfModelInThisBundle` to create a MLModel object to pass-in.

        - parameters:
          - model: MLModel object
    */
    init(model: MLModel) {
        self.model = model
    }

    /**
        Construct DictVec2_int instance by automatically loading the model from the app's bundle.
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
    @available(macOS 10.14, iOS 12.0, tvOS 12.0, watchOS 5.0, visionOS 1.0, *)
    public convenience init(configuration: MLModelConfiguration) throws {
        try self.init(contentsOf: type(of:self).urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct DictVec2_int instance with explicit path to mlmodelc file
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
    @available(macOS 10.14, iOS 12.0, tvOS 12.0, watchOS 5.0, visionOS 1.0, *)
    public convenience init(contentsOf modelURL: URL, configuration: MLModelConfiguration) throws {
        try self.init(model: MLModel(contentsOf: modelURL, configuration: configuration))
    }

    /**
        Construct DictVec2_int instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    @available(macOS 11.0, iOS 14.0, tvOS 14.0, watchOS 7.0, visionOS 1.0, *)
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<DictVec2_int, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct DictVec2_int instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    @available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> DictVec2_int {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct DictVec2_int instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    @available(macOS 11.0, iOS 14.0, tvOS 14.0, watchOS 7.0, visionOS 1.0, *)
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<DictVec2_int, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(DictVec2_int(model: model)))
            }
        }
    }

    /**
        Construct DictVec2_int instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    @available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> DictVec2_int {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return DictVec2_int(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as DictVec2_intInput

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as DictVec2_intOutput
    */
    public func prediction(input: DictVec2_intInput) throws -> DictVec2_intOutput {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as DictVec2_intInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as DictVec2_intOutput
    */
    public func prediction(input: DictVec2_intInput, options: MLPredictionOptions) throws -> DictVec2_intOutput {
        let outFeatures = try model.prediction(from: input, options: options)
        return DictVec2_intOutput(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as DictVec2_intInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as DictVec2_intOutput
    */
    @available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
    public func prediction(input: DictVec2_intInput, options: MLPredictionOptions = MLPredictionOptions()) async throws -> DictVec2_intOutput {
        let outFeatures = try await model.prediction(from: input, options: options)
        return DictVec2_intOutput(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - x: dictionary of 64-bit integers to doubles

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as DictVec2_intOutput
    */
    public func prediction(x: [Int64 : Double]) throws -> DictVec2_intOutput {
        let input_ = DictVec2_intInput(x: x)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [DictVec2_intInput]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [DictVec2_intOutput]
    */
    @available(macOS 10.14, iOS 12.0, tvOS 12.0, watchOS 5.0, visionOS 1.0, *)
    public func predictions(inputs: [DictVec2_intInput], options: MLPredictionOptions = MLPredictionOptions()) throws -> [DictVec2_intOutput] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [DictVec2_intOutput] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  DictVec2_intOutput(features: outProvider)
            results.append(result)
        }
        return results
    }
}
