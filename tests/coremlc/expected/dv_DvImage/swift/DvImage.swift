//
// DvImage.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
public class DvImageInput : MLFeatureProvider {

    /// img as color (kCVPixelFormatType_32BGRA) image buffer, 4 pixels wide by 4 pixels high
    public var img: CVPixelBuffer

    /// bias as optional 1 × 3 × 4 × 4 4-dimensional array of floats
    public var bias: MLMultiArray? = nil

    public var featureNames: Set<String> { ["img", "bias"] }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        if featureName == "img" {
            return MLFeatureValue(pixelBuffer: img)
        }
        if featureName == "bias" {
            return bias == nil ? nil : MLFeatureValue(multiArray: bias!)
        }
        return nil
    }

    public init(img: CVPixelBuffer, bias: MLMultiArray? = nil) {
        self.img = img
        self.bias = bias
    }

    public convenience init(img: CVPixelBuffer, bias: MLShapedArray<Float>? = nil) {
        self.init(img: img, bias: bias != nil ? MLMultiArray(bias!) : nil)
    }

    public convenience init(imgWith img: CGImage, bias: MLMultiArray? = nil) throws {
        self.init(img: try MLFeatureValue(cgImage: img, pixelsWide: 4, pixelsHigh: 4, pixelFormatType: kCVPixelFormatType_32ARGB, options: nil).imageBufferValue!, bias: bias)
    }

    public convenience init(imgWith img: CGImage, bias: MLShapedArray<Float>? = nil) throws {
        self.init(img: try MLFeatureValue(cgImage: img, pixelsWide: 4, pixelsHigh: 4, pixelFormatType: kCVPixelFormatType_32ARGB, options: nil).imageBufferValue!, bias: bias != nil ? MLMultiArray(bias!) : nil)
    }

    public convenience init(imgAt img: URL, bias: MLMultiArray? = nil) throws {
        self.init(img: try MLFeatureValue(imageAt: img, pixelsWide: 4, pixelsHigh: 4, pixelFormatType: kCVPixelFormatType_32ARGB, options: nil).imageBufferValue!, bias: bias)
    }

    public convenience init(imgAt img: URL, bias: MLShapedArray<Float>? = nil) throws {
        self.init(img: try MLFeatureValue(imageAt: img, pixelsWide: 4, pixelsHigh: 4, pixelFormatType: kCVPixelFormatType_32ARGB, options: nil).imageBufferValue!, bias: bias != nil ? MLMultiArray(bias!) : nil)
    }

    public func setImg(with img: CGImage) throws  {
        self.img = try MLFeatureValue(cgImage: img, pixelsWide: 4, pixelsHigh: 4, pixelFormatType: kCVPixelFormatType_32ARGB, options: nil).imageBufferValue!
    }

    public func setImg(with img: URL) throws  {
        self.img = try MLFeatureValue(imageAt: img, pixelsWide: 4, pixelsHigh: 4, pixelFormatType: kCVPixelFormatType_32ARGB, options: nil).imageBufferValue!
    }

}


/// Model Prediction Output Type
@available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
public class DvImageOutput : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// add_0 as 1 × 3 × 4 × 4 4-dimensional array of floats
    public var add_0: MLMultiArray {
        provider.featureValue(for: "add_0")!.multiArrayValue!
    }

    /// add_0 as 1 × 3 × 4 × 4 4-dimensional array of floats
    public var add_0ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(add_0)
    }

    public var featureNames: Set<String> {
        provider.featureNames
    }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        provider.featureValue(for: featureName)
    }

    public init(add_0: MLMultiArray) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["add_0" : MLFeatureValue(multiArray: add_0)])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}


/// Class for model loading and prediction
@available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
public class DvImage {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "DvImage", withExtension:"mlmodelc")!
    }

    /**
        Construct DvImage instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of DvImage.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `DvImage.urlOfModelInThisBundle` to create a MLModel object to pass-in.

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
        Construct DvImage instance with explicit path to mlmodelc file
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
        Construct DvImage instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<DvImage, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct DvImage instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> DvImage {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct DvImage instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<DvImage, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(DvImage(model: model)))
            }
        }
    }

    /**
        Construct DvImage instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> DvImage {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return DvImage(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as DvImageInput

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as DvImageOutput
    */
    public func prediction(input: DvImageInput) throws -> DvImageOutput {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as DvImageInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as DvImageOutput
    */
    public func prediction(input: DvImageInput, options: MLPredictionOptions) throws -> DvImageOutput {
        let outFeatures = try model.prediction(from: input, options: options)
        return DvImageOutput(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as DvImageInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as DvImageOutput
    */
    public func prediction(input: DvImageInput, options: MLPredictionOptions = MLPredictionOptions()) async throws -> DvImageOutput {
        let outFeatures = try await model.prediction(from: input, options: options)
        return DvImageOutput(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - img: color (kCVPixelFormatType_32BGRA) image buffer, 4 pixels wide by 4 pixels high
            - bias: optional 1 × 3 × 4 × 4 4-dimensional array of floats

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as DvImageOutput
    */
    public func prediction(img: CVPixelBuffer, bias: MLMultiArray?) throws -> DvImageOutput {
        let input_ = DvImageInput(img: img, bias: bias)
        return try prediction(input: input_)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - img: color (kCVPixelFormatType_32BGRA) image buffer, 4 pixels wide by 4 pixels high
            - bias: optional 1 × 3 × 4 × 4 4-dimensional array of floats

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as DvImageOutput
    */

    public func prediction(img: CVPixelBuffer, bias: MLShapedArray<Float>?) throws -> DvImageOutput {
        let input_ = DvImageInput(img: img, bias: bias)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [DvImageInput]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [DvImageOutput]
    */
    public func predictions(inputs: [DvImageInput], options: MLPredictionOptions = MLPredictionOptions()) throws -> [DvImageOutput] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [DvImageOutput] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  DvImageOutput(features: outProvider)
            results.append(result)
        }
        return results
    }
}
