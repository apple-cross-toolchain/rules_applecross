//
// ImNonSqPlus.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 13.0, iOS 16.0, tvOS 16.0, watchOS 9.0, visionOS 1.0, *)
public class ImNonSqPlusInput : MLFeatureProvider {

    /// photo as color (kCVPixelFormatType_32BGRA) image buffer, 10 pixels wide by 6 pixels high
    public var photo: CVPixelBuffer

    /// k as 1 by 1 matrix of floats
    public var k: MLMultiArray

    public var featureNames: Set<String> { ["photo", "k"] }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        if featureName == "photo" {
            return MLFeatureValue(pixelBuffer: photo)
        }
        if featureName == "k" {
            return MLFeatureValue(multiArray: k)
        }
        return nil
    }

    public init(photo: CVPixelBuffer, k: MLMultiArray) {
        self.photo = photo
        self.k = k
    }

    public convenience init(photo: CVPixelBuffer, k: MLShapedArray<Float>) {
        self.init(photo: photo, k: MLMultiArray(k))
    }

    public convenience init(photoWith photo: CGImage, k: MLMultiArray) throws {
        self.init(photo: try MLFeatureValue(cgImage: photo, pixelsWide: 10, pixelsHigh: 6, pixelFormatType: kCVPixelFormatType_32BGRA, options: nil).imageBufferValue!, k: k)
    }

    public convenience init(photoWith photo: CGImage, k: MLShapedArray<Float>) throws {
        self.init(photo: try MLFeatureValue(cgImage: photo, pixelsWide: 10, pixelsHigh: 6, pixelFormatType: kCVPixelFormatType_32BGRA, options: nil).imageBufferValue!, k: MLMultiArray(k))
    }

    public convenience init(photoAt photo: URL, k: MLMultiArray) throws {
        self.init(photo: try MLFeatureValue(imageAt: photo, pixelsWide: 10, pixelsHigh: 6, pixelFormatType: kCVPixelFormatType_32BGRA, options: nil).imageBufferValue!, k: k)
    }

    public convenience init(photoAt photo: URL, k: MLShapedArray<Float>) throws {
        self.init(photo: try MLFeatureValue(imageAt: photo, pixelsWide: 10, pixelsHigh: 6, pixelFormatType: kCVPixelFormatType_32BGRA, options: nil).imageBufferValue!, k: MLMultiArray(k))
    }

    public func setPhoto(with photo: CGImage) throws  {
        self.photo = try MLFeatureValue(cgImage: photo, pixelsWide: 10, pixelsHigh: 6, pixelFormatType: kCVPixelFormatType_32BGRA, options: nil).imageBufferValue!
    }

    public func setPhoto(with photo: URL) throws  {
        self.photo = try MLFeatureValue(imageAt: photo, pixelsWide: 10, pixelsHigh: 6, pixelFormatType: kCVPixelFormatType_32BGRA, options: nil).imageBufferValue!
    }

}


/// Model Prediction Output Type
@available(macOS 13.0, iOS 16.0, tvOS 16.0, watchOS 9.0, visionOS 1.0, *)
public class ImNonSqPlusOutput : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// reshape_0 as 1 element vector of floats
    public var reshape_0: MLMultiArray {
        provider.featureValue(for: "reshape_0")!.multiArrayValue!
    }

    /// reshape_0 as 1 element vector of floats
    public var reshape_0ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(reshape_0)
    }

    public var featureNames: Set<String> {
        provider.featureNames
    }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        provider.featureValue(for: featureName)
    }

    public init(reshape_0: MLMultiArray) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["reshape_0" : MLFeatureValue(multiArray: reshape_0)])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}


/// Class for model loading and prediction
@available(macOS 13.0, iOS 16.0, tvOS 16.0, watchOS 9.0, visionOS 1.0, *)
public class ImNonSqPlus {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "ImNonSqPlus", withExtension:"mlmodelc")!
    }

    /**
        Construct ImNonSqPlus instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of ImNonSqPlus.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `ImNonSqPlus.urlOfModelInThisBundle` to create a MLModel object to pass-in.

        - parameters:
          - model: MLModel object
    */
    init(model: MLModel) {
        self.model = model
    }

    /**
        Construct a model with configuration

        - parameters:
           - configuration: the desired model configuration

        - throws: an NSError object that describes the problem
    */
    public convenience init(configuration: MLModelConfiguration = MLModelConfiguration()) throws {
        try self.init(contentsOf: type(of:self).urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct ImNonSqPlus instance with explicit path to mlmodelc file
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
        Construct ImNonSqPlus instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<ImNonSqPlus, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct ImNonSqPlus instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> ImNonSqPlus {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct ImNonSqPlus instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<ImNonSqPlus, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(ImNonSqPlus(model: model)))
            }
        }
    }

    /**
        Construct ImNonSqPlus instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> ImNonSqPlus {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return ImNonSqPlus(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as ImNonSqPlusInput

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as ImNonSqPlusOutput
    */
    public func prediction(input: ImNonSqPlusInput) throws -> ImNonSqPlusOutput {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as ImNonSqPlusInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as ImNonSqPlusOutput
    */
    public func prediction(input: ImNonSqPlusInput, options: MLPredictionOptions) throws -> ImNonSqPlusOutput {
        let outFeatures = try model.prediction(from: input, options: options)
        return ImNonSqPlusOutput(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as ImNonSqPlusInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as ImNonSqPlusOutput
    */
    @available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
    public func prediction(input: ImNonSqPlusInput, options: MLPredictionOptions = MLPredictionOptions()) async throws -> ImNonSqPlusOutput {
        let outFeatures = try await model.prediction(from: input, options: options)
        return ImNonSqPlusOutput(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - photo: color (kCVPixelFormatType_32BGRA) image buffer, 10 pixels wide by 6 pixels high
            - k: 1 by 1 matrix of floats

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as ImNonSqPlusOutput
    */
    public func prediction(photo: CVPixelBuffer, k: MLMultiArray) throws -> ImNonSqPlusOutput {
        let input_ = ImNonSqPlusInput(photo: photo, k: k)
        return try prediction(input: input_)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - photo: color (kCVPixelFormatType_32BGRA) image buffer, 10 pixels wide by 6 pixels high
            - k: 1 by 1 matrix of floats

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as ImNonSqPlusOutput
    */

    public func prediction(photo: CVPixelBuffer, k: MLShapedArray<Float>) throws -> ImNonSqPlusOutput {
        let input_ = ImNonSqPlusInput(photo: photo, k: k)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [ImNonSqPlusInput]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [ImNonSqPlusOutput]
    */
    public func predictions(inputs: [ImNonSqPlusInput], options: MLPredictionOptions = MLPredictionOptions()) throws -> [ImNonSqPlusOutput] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [ImNonSqPlusOutput] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  ImNonSqPlusOutput(features: outProvider)
            results.append(result)
        }
        return results
    }
}
