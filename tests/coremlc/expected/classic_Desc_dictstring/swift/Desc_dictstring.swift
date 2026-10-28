//
// Desc_dictstring.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 10.15, iOS 13.0, tvOS 13.0, watchOS 6.0, visionOS 1.0, *)
public class Desc_dictstringInput : MLFeatureProvider {

    /// some text as dictionary of strings to doubles
    public var i: [String : Double]

    public var featureNames: Set<String> { ["i"] }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        if featureName == "i" {
            return try! MLFeatureValue(dictionary: i as [NSObject : NSNumber])
        }
        return nil
    }

    public init(i: [String : Double]) {
        self.i = i
    }

}


/// Model Prediction Output Type
@available(macOS 10.15, iOS 13.0, tvOS 13.0, watchOS 6.0, visionOS 1.0, *)
public class Desc_dictstringOutput : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// some text as dictionary of strings to doubles
    public var o: [String : Double] {
        provider.featureValue(for: "o")!.dictionaryValue as! [String : Double]
    }

    public var featureNames: Set<String> {
        provider.featureNames
    }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        provider.featureValue(for: featureName)
    }

    public init(o: [String : Double]) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["o" : MLFeatureValue(dictionary: o as [AnyHashable : NSNumber])])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}


/// Class for model loading and prediction
@available(macOS 10.15, iOS 13.0, tvOS 13.0, watchOS 6.0, visionOS 1.0, *)
public class Desc_dictstring {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "Desc_dictstring", withExtension:"mlmodelc")!
    }

    /**
        Construct Desc_dictstring instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of Desc_dictstring.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `Desc_dictstring.urlOfModelInThisBundle` to create a MLModel object to pass-in.

        - parameters:
          - model: MLModel object
    */
    init(model: MLModel) {
        self.model = model
    }

    /**
        Construct Desc_dictstring instance by automatically loading the model from the app's bundle.
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
        Construct Desc_dictstring instance with explicit path to mlmodelc file
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
        Construct Desc_dictstring instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    @available(macOS 11.0, iOS 14.0, tvOS 14.0, watchOS 7.0, visionOS 1.0, *)
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<Desc_dictstring, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct Desc_dictstring instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    @available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> Desc_dictstring {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct Desc_dictstring instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    @available(macOS 11.0, iOS 14.0, tvOS 14.0, watchOS 7.0, visionOS 1.0, *)
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<Desc_dictstring, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(Desc_dictstring(model: model)))
            }
        }
    }

    /**
        Construct Desc_dictstring instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    @available(macOS 12.0, iOS 15.0, tvOS 15.0, watchOS 8.0, visionOS 1.0, *)
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> Desc_dictstring {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return Desc_dictstring(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as Desc_dictstringInput

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as Desc_dictstringOutput
    */
    public func prediction(input: Desc_dictstringInput) throws -> Desc_dictstringOutput {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as Desc_dictstringInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as Desc_dictstringOutput
    */
    public func prediction(input: Desc_dictstringInput, options: MLPredictionOptions) throws -> Desc_dictstringOutput {
        let outFeatures = try model.prediction(from: input, options: options)
        return Desc_dictstringOutput(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as Desc_dictstringInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as Desc_dictstringOutput
    */
    @available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
    public func prediction(input: Desc_dictstringInput, options: MLPredictionOptions = MLPredictionOptions()) async throws -> Desc_dictstringOutput {
        let outFeatures = try await model.prediction(from: input, options: options)
        return Desc_dictstringOutput(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - i: some text as dictionary of strings to doubles

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as Desc_dictstringOutput
    */
    public func prediction(i: [String : Double]) throws -> Desc_dictstringOutput {
        let input_ = Desc_dictstringInput(i: i)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [Desc_dictstringInput]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [Desc_dictstringOutput]
    */
    public func predictions(inputs: [Desc_dictstringInput], options: MLPredictionOptions = MLPredictionOptions()) throws -> [Desc_dictstringOutput] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [Desc_dictstringOutput] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  Desc_dictstringOutput(features: outProvider)
            results.append(result)
        }
        return results
    }
}
